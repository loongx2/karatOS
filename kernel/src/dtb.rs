//! ============================================================================
//! MODULE : dtb — runtime flat device tree (FDT/DTB) reader — Phase 2
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   Replaces compile-time device tables with a RUNTIME walk of the device
//!   tree blob that firmware/QEMU hands the kernel. On RISC-V the boot
//!   protocol passes the DTB pointer in `a1` (captured by the entry point);
//!   boards without a firmware DTB (ARM LM3S under QEMU) fall back to the
//!   built-in table in board.rs — the "config from OTP/flash with safe
//!   fallback" model.
//!
//! SCOPE (deliberately minimal — this is an RTOS, not Linux)
//!   * header validation (magic, spec version, sane sizes)
//!   * structure-block token walk (BEGIN_NODE/END_NODE/PROP/NOP/END)
//!   * per node: `compatible`, `reg` base, `status`, `clock-frequency`
//!   * `#address-cells` honored per spec (default 2)
//!   Everything else (interrupt-parents, ranges, phandles) is Phase 4/5.
//!
//! MEMORY BUDGET
//!   Zero-copy: the parser holds a `&[u8]` view of the firmware-owned blob
//!   (never mutated, alive for the kernel's lifetime). Output is a bounded
//!   `heapless::Vec<DeviceConfig, 8>` — no allocation.
//!
//! HOST TESTS
//!   A synthetic blob is built byte-by-byte below and walked by the exact
//!   code paths used on target (`tests` module at the bottom).
//! ============================================================================

use crate::drivers::{DeviceClass, DeviceConfig, DriverFlavor};

/// FDT magic ("d00dfeed", big-endian on the wire).
pub const FDT_MAGIC: u32 = 0xD00D_FEED;

// Structure-block tokens (big-endian u32 on the wire).
const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE: u32 = 0x2;
const FDT_PROP: u32 = 0x3;
const FDT_NOP: u32 = 0x4;
const FDT_END: u32 = 0x9;

/// A validated DTB view. `data` covers the whole blob.
#[derive(Debug)]
pub struct Fdt {
    data: &'static [u8],
    struct_off: usize,
    strings_off: usize,
    /// Total blob size (needed to place the module store after it in Phase 4).
    pub total_size: usize,
}

impl Fdt {
    /// Probe a candidate address: validates magic + header sanity and
    /// returns a parser view. `None` when no plausible DTB lives there.
    pub fn probe(addr: usize) -> Option<Fdt> {
        if addr == 0 {
            return None;
        }
        // SAFETY: aligned 32-bit probe read at a caller-supplied boot address.
        let magic = unsafe { (addr as *const u32).read_volatile() }.to_be();
        if magic != FDT_MAGIC {
            return None;
        }
        // SAFETY: valid magic implies a real 40-byte FDT header.
        let hdr = unsafe { core::slice::from_raw_parts(addr as *const u8, 40) };
        let be = |off: usize| -> usize {
            u32::from_be_bytes([hdr[off], hdr[off + 1], hdr[off + 2], hdr[off + 3]]) as usize
        };
        let total_size = be(4);
        let struct_off = be(8);
        let strings_off = be(12);
        let version = be(20);
        if !(40..=1024 * 1024).contains(&total_size) {
            return None; // implausible size: not a DTB
        }
        if !(16..=20).contains(&version) {
            return None; // only spec versions we understand
        }
        if struct_off >= total_size || strings_off >= total_size {
            return None;
        }
        // SAFETY: size validated; the region is firmware-owned RAM/flash that
        // stays mapped and unmodified for the kernel lifetime, so the
        // 'static borrow holds by platform contract.
        let data = unsafe { core::slice::from_raw_parts(addr as *const u8, total_size) };
        Some(Fdt {
            data,
            struct_off,
            strings_off,
            total_size,
        })
    }

    fn be32(&self, off: usize) -> u32 {
        u32::from_be_bytes([
            self.data[off],
            self.data[off + 1],
            self.data[off + 2],
            self.data[off + 3],
        ])
    }

    /// Walk the structure block and enumerate karatOS-supported devices:
    /// nodes whose `compatible` maps to a driver flavor and whose `status`
    /// is not `disabled`. One driver per class (first match wins).
    pub fn device_configs(&self) -> heapless::Vec<DeviceConfig, 8> {
        let mut w = Walker {
            fdt: self as *const Fdt,
            cells: [2; 8], // spec default #address-cells = 2
            depth: 0,
            compat: heapless::String::new(),
            status_ok: true,
            base: None,
            clock: 0,
            has_device: false,
            flavor: DriverFlavor::Pl011,
            out: heapless::Vec::new(),
        };
        w.run();
        w.out
    }

    fn string(&self, nameoff: usize) -> &str {
        let start = self.strings_off + nameoff;
        let end = start
            + self
                .data
                .get(start..)
                .unwrap_or(&[])
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(0);
        core::str::from_utf8(&self.data[start..end]).unwrap_or("?")
    }
}

/// Map a `compatible` string to a driver flavor, if the kernel supports it.
pub fn flavor_of(compatible: &str) -> Option<DriverFlavor> {
    match compatible {
        "arm,pl011" | "arm,primecell" => Some(DriverFlavor::Pl011),
        "ns16550a" | "ns16550" => Some(DriverFlavor::Ns16550),
        "riscv,clint0" | "sifive,clint0" | "riscv,clint" => Some(DriverFlavor::Clint),
        "arm,armv7-timer" => Some(DriverFlavor::SysTick),
        _ => None,
    }
}

/// `reg` base extraction: first `addr_cells` big-endian words, collapsed to
/// a `usize` (all supported targets place devices below 4 GiB).
fn reg_base(value: &[u8], addr_cells: usize) -> Option<usize> {
    let mut base: u64 = 0;
    for (i, w) in value.chunks_exact(4).enumerate() {
        if i >= addr_cells {
            break;
        }
        base = (base << 32) | u32::from_be_bytes([w[0], w[1], w[2], w[3]]) as u64;
    }
    usize::try_from(base).ok()
}

/// Single-pass stateful walker: accumulates per-node properties and commits
/// a `DeviceConfig` when a node closes (FDT_END_NODE).
struct Walker {
    fdt: *const Fdt,
    /// `#address-cells` in effect for the node at index `depth`.
    cells: [usize; 8],
    depth: usize,
    // Per-node accumulation (reset on BEGIN_NODE).
    compat: heapless::String<64>,
    status_ok: bool,
    base: Option<usize>,
    clock: u32,
    has_device: bool,
    flavor: DriverFlavor,
    out: heapless::Vec<DeviceConfig, 8>,
}

impl Walker {
    fn fdt(&self) -> &Fdt {
        // SAFETY: `fdt` is set once at construction and never mutated.
        unsafe { &*self.fdt }
    }

    fn run(&mut self) {
        let struct_off = self.fdt().struct_off;
        let mut off = struct_off;
        loop {
            let data_len = self.fdt().data.len();
            if off + 4 > data_len {
                return; // truncated blob: keep what we collected
            }
            let token = self.fdt().be32(off);
            off += 4;
            match token {
                FDT_BEGIN_NODE => {
                    let mut end = off;
                    while end < data_len && self.fdt().data[end] != 0 {
                        end += 1;
                    }
                    off = (end + 1 + 3) & !3;
                    self.depth = (self.depth + 1).min(self.cells.len() - 1);
                    self.begin_node();
                }
                FDT_END_NODE => {
                    self.end_node();
                    self.depth = self.depth.saturating_sub(1);
                }
                FDT_PROP => {
                    if off + 8 > data_len {
                        return;
                    }
                    // Gather property coordinates with short immutable
                    // borrows, copy the bytes out, THEN mutate: NLL ends
                    // the self borrow before on_prop(&mut self).
                    let (len, nameoff, val) = {
                        let f = self.fdt();
                        (f.be32(off) as usize, f.be32(off + 4) as usize, off + 8)
                    };
                    let next = (val + len + 3) & !3;
                    if next > data_len || val + len > data_len {
                        return;
                    }
                    let mut pbuf = [0u8; 24];
                    let mut vbuf = [0u8; 64];
                    {
                        let f = self.fdt();
                        let ps = f.strings_off + nameoff;
                        let pe = ps + f.data[ps..]
                            .iter()
                            .position(|&b| b == 0)
                            .unwrap_or(0);
                        let n = (pe - ps).min(pbuf.len());
                        pbuf[..n].copy_from_slice(&f.data[ps..ps + n]);
                        let vn = len.min(vbuf.len());
                        vbuf[..vn].copy_from_slice(&f.data[val..val + vn]);
                    }
                    off = next;
                    let prop_len = pbuf.iter().position(|&b| b == 0).unwrap_or(pbuf.len());
                    let prop = core::str::from_utf8(&pbuf[..prop_len]).unwrap_or("?");
                    self.on_prop(prop, &vbuf[..len.min(vbuf.len())]);
                }
                FDT_NOP => {}
                FDT_END => return,
                _ => return, // corrupt stream: keep what we collected
            }
        }
    }

    fn begin_node(&mut self) {
        if self.depth >= 1 {
            self.compat.clear();
            self.status_ok = true;
            self.base = None;
            self.clock = 0;
            self.has_device = false;
        }
    }

    fn on_prop(&mut self, prop: &str, value: &[u8]) {
        match prop {
            // Applies to the CHILDREN of the node carrying it.
            "#address-cells" if value.len() == 4 => {
                let cells =
                    u32::from_be_bytes([value[0], value[1], value[2], value[3]]) as usize;
                if (2..self.cells.len()).contains(&(self.depth + 1)) {
                    self.cells[self.depth + 1] = cells.clamp(1, 2);
                }
            }
            "compatible" => {
                self.compat.clear();
                let end = value
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(value.len()); // defensive: non-terminated value
                let _ = self
                    .compat
                    .push_str(core::str::from_utf8(&value[..end]).unwrap_or(""));
                if let Some(flavor) = flavor_of(self.compat.as_str()) {
                    self.has_device = true;
                    self.flavor = flavor;
                }
            }
            "status" => self.status_ok = value != b"disabled",
            "clock-frequency" if value.len() == 4 => {
                self.clock = u32::from_be_bytes([value[0], value[1], value[2], value[3]]);
            }
            "reg" => self.base = reg_base(value, self.cells[self.depth]),
            _ => {}
        }
    }

    fn end_node(&mut self) {
        if self.depth == 0 || !self.has_device || !self.status_ok {
            return;
        }
        let Some(base) = self.base else {
            return;
        };
        // One driver per class: the FIRST match wins (console/tick source).
        let class = DeviceClass::from_flavor(self.flavor);
        if self.out.iter().any(|c| c.class == class) {
            return;
        }
        let name = match class {
            DeviceClass::Uart => "uart0",
            DeviceClass::Timer => "timer0",
        };
        let _ = self.out.push(DeviceConfig {
            name,
            class,
            flavor: self.flavor,
            base,
            clock_hz: self.clock,
            baud: Some(115_200),
        });
    }
}
// ---------------------------------------------------------------------------
// Host tests — synthetic blob built byte-by-byte, walked by the same code
// paths used on target.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn be32v(v: u32) -> heapless::Vec<u8, 8> {
        let mut b = heapless::Vec::new();
        b.extend_from_slice(&v.to_be_bytes()).unwrap();
        b
    }

    /// Append `s` + NUL terminator, then pad to 4 bytes — exactly how FDT
    /// stores names and NUL-terminated string property values.
    fn padded(blob: &mut heapless::Vec<u8, 1024>, s: &[u8]) {
        blob.extend_from_slice(s).unwrap();
        blob.push(0).unwrap();
        while blob.len() % 4 != 0 {
            blob.push(0).unwrap();
        }
    }

    /// / { #address-cells=<1>; compatible="test-root";
    ///     uart@10000000  { compatible="ns16550a"; reg=<0x10000000 0x100>;
    ///                      clock-frequency=<10000000>; };
    ///     clint@2000000  { compatible="riscv,clint0"; reg=<0x02000000 0x10000>; };
    ///     dead@3         { compatible="ns16550a"; status="disabled"; }; }
    fn synthetic_blob() -> heapless::Vec<u8, 1024> {
        let mut blob = heapless::Vec::new();
        blob.extend_from_slice(&[0u8; 40]).unwrap(); // header placeholder
        let strings_off = 40u32;
        let mut strings = heapless::Vec::<u8, 256>::new();
        let mut stroff = |strings: &mut heapless::Vec<u8, 256>, s: &str| -> u32 {
            let off = strings.len() as u32;
            strings.extend_from_slice(s.as_bytes()).unwrap();
            strings.push(0).unwrap();
            off
        };
        let s_cells = stroff(&mut strings, "#address-cells");
        let s_compat = stroff(&mut strings, "compatible");
        let s_reg = stroff(&mut strings, "reg");
        let s_clock = stroff(&mut strings, "clock-frequency");
        let s_status = stroff(&mut strings, "status");

        let mut st = heapless::Vec::<u8, 1024>::new();
        // root
        st.extend_from_slice(&be32v(FDT_BEGIN_NODE)).unwrap();
        padded(&mut st, b"\0"); // empty root name is one NUL byte, padded
        st.extend_from_slice(&be32v(FDT_PROP)).unwrap();
        st.extend_from_slice(&be32v(4)).unwrap();
        st.extend_from_slice(&be32v(s_cells)).unwrap();
        st.extend_from_slice(&1u32.to_be_bytes()).unwrap();
        // uart@10000000 (enabled, with clock)
        st.extend_from_slice(&be32v(FDT_BEGIN_NODE)).unwrap();
        padded(&mut st, b"uart@10000000");
        // compatible (len=9, NUL-terminated string, padded)
        st.extend_from_slice(&be32v(FDT_PROP)).unwrap();
        st.extend_from_slice(&be32v(9)).unwrap();
        st.extend_from_slice(&be32v(s_compat)).unwrap();
        padded(&mut st, b"ns16550a\0");
        // reg = <0x10000000 0x100> (len=8: address + size cells)
        st.extend_from_slice(&be32v(FDT_PROP)).unwrap();
        st.extend_from_slice(&be32v(8)).unwrap();
        st.extend_from_slice(&be32v(s_reg)).unwrap();
        st.extend_from_slice(&0x1000_0000u32.to_be_bytes()).unwrap();
        st.extend_from_slice(&0x100u32.to_be_bytes()).unwrap();
        // clock-frequency (len=4)
        st.extend_from_slice(&be32v(FDT_PROP)).unwrap();
        st.extend_from_slice(&be32v(4)).unwrap();
        st.extend_from_slice(&be32v(s_clock)).unwrap();
        st.extend_from_slice(&10_000_000u32.to_be_bytes()).unwrap();
        st.extend_from_slice(&be32v(FDT_END_NODE)).unwrap();
        // clint@2000000
        st.extend_from_slice(&be32v(FDT_BEGIN_NODE)).unwrap();
        padded(&mut st, b"clint@2000000");
        st.extend_from_slice(&be32v(FDT_PROP)).unwrap();
        st.extend_from_slice(&be32v(13)).unwrap();
        st.extend_from_slice(&be32v(s_compat)).unwrap();
        padded(&mut st, b"riscv,clint0");
        st.extend_from_slice(&be32v(FDT_PROP)).unwrap();
        st.extend_from_slice(&be32v(8)).unwrap();
        st.extend_from_slice(&be32v(s_reg)).unwrap();
        st.extend_from_slice(&0x0200_0000u32.to_be_bytes()).unwrap();
        st.extend_from_slice(&0x1_0000u32.to_be_bytes()).unwrap();
        st.extend_from_slice(&be32v(FDT_END_NODE)).unwrap();
        // dead@3 (disabled — must be skipped)
        st.extend_from_slice(&be32v(FDT_BEGIN_NODE)).unwrap();
        padded(&mut st, b"dead@3");
        st.extend_from_slice(&be32v(FDT_PROP)).unwrap();
        st.extend_from_slice(&be32v(8)).unwrap();
        st.extend_from_slice(&be32v(s_compat)).unwrap();
        padded(&mut st, b"ns16550a");
        st.extend_from_slice(&be32v(FDT_PROP)).unwrap();
        st.extend_from_slice(&be32v(8)).unwrap();
        st.extend_from_slice(&be32v(s_status)).unwrap();
        padded(&mut st, b"disabled");
        st.extend_from_slice(&be32v(FDT_END_NODE)).unwrap();
        // close root + blob
        st.extend_from_slice(&be32v(FDT_END_NODE)).unwrap();
        st.extend_from_slice(&be32v(FDT_END)).unwrap();

        blob.extend_from_slice(&strings).unwrap();
        // FDT spec: the structure block starts 4-byte aligned after strings.
        while blob.len() % 4 != 0 {
            blob.push(0).unwrap();
        }
        let struct_off = blob.len(); // AFTER padding — header must agree
        blob.extend_from_slice(&st).unwrap();
        let total = blob.len() as u32;
        blob[0..4].copy_from_slice(&FDT_MAGIC.to_be_bytes());
        blob[4..8].copy_from_slice(&total.to_be_bytes());
        blob[8..12].copy_from_slice(&(struct_off as u32).to_be_bytes());
        blob[12..16].copy_from_slice(&strings_off.to_be_bytes());
        blob[16..20].copy_from_slice(&0u32.to_be_bytes());
        blob[20..24].copy_from_slice(&17u32.to_be_bytes());
        blob[24..28].copy_from_slice(&0u32.to_be_bytes());
        blob[28..32].copy_from_slice(&(strings.len() as u32).to_be_bytes());
        blob[32..36].copy_from_slice(&(st.len() as u32).to_be_bytes());
        blob
    }

#[test]
    fn dump_blob_bytes() {
        let blob = synthetic_blob();
        for (i, chunk) in blob.chunks(4).enumerate() {
            std::println!("blob[{:3}..{:3}] = {:02x?} (be32={})", i*4, (i+1)*4, chunk, u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
    }

    #[test]
    fn probe_rejects_garbage_and_accepts_blob() {
        let blob = synthetic_blob();
        let addr = blob.as_ptr() as usize;
        assert!(Fdt::probe(0).is_none());
        let junk = [0u8; 64];
        assert!(Fdt::probe(junk.as_ptr() as usize).is_none());
        let fdt = Fdt::probe(addr).expect("valid blob should probe");
        assert_eq!(fdt.total_size, blob.len());
    }

    #[test]
    fn enumerates_supported_devices_with_base_and_clock() {
        let blob = synthetic_blob();
        let fdt = Fdt::probe(blob.as_ptr() as usize).unwrap();
        let devs = fdt.device_configs();
        assert_eq!(devs.len(), 2, "uart + timer; disabled node skipped");
        assert_eq!(devs[0].name, "uart0");
        assert_eq!(devs[0].class, DeviceClass::Uart);
        assert_eq!(devs[0].flavor, DriverFlavor::Ns16550);
        assert_eq!(devs[0].base, 0x1000_0000);
        assert_eq!(devs[0].clock_hz, 10_000_000);
        assert_eq!(devs[1].name, "timer0");
        assert_eq!(devs[1].flavor, DriverFlavor::Clint);
        assert_eq!(devs[1].base, 0x0200_0000);
    }

    #[test]
    fn flavor_mapping() {
        assert_eq!(flavor_of("arm,pl011"), Some(DriverFlavor::Pl011));
        assert_eq!(flavor_of("ns16550a"), Some(DriverFlavor::Ns16550));
        assert_eq!(flavor_of("riscv,clint0"), Some(DriverFlavor::Clint));
        assert_eq!(flavor_of("acme,unknown"), None);
    }
}
