//! ============================================================================
//! MODULE : store — flash module store driver (Phase 4)
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   The store is a DEDICATED, fixed-address region where provisioned module
//!   images live (flash on ARM, a bootloader/QEMU-provisioned region on
//!   RISC-V until Phase 5 maps pflash). This module:
//!     * probes and validates the store (magic, layout version, BOARD ID,
//!       CRC32 over directory + images),
//!     * looks modules up BY NAME in the directory — the kernel no longer
//!       needs to know which modules exist at compile time,
//!     * hands verified image slices to the loader (modules::load_image),
//!       which copies them to the RAM slot and starts them.
//!
//! BOARD ID (the OTP stand-in)
//!   The store header carries the board it was provisioned for. The kernel
//!   compares it against `board::BOARD_ID` (read from OTP on real silicon;
//!   known-by-construction in QEMU). A mismatched store is refused before
//!   any module runs; board-id 0 = unprovisioned wildcard.
//!
//! CONCURRENCY
//!   Main-loop context only (same contract as modules.rs / drivers.rs).
//!
//! MEMORY BUDGET
//!   Zero-copy: validated slices into the store region. Code ~1 kB ROM.
//! ============================================================================

use core::ptr;

use crate::board;
pub use karatos_kapi as abi;

use abi::{StoreEntry, StoreHeader};

/// Per-target store base address.
#[cfg(all(target_arch = "arm", armv8m_target))]
const STORE_BASE: usize = abi::STORE_BASE_ARMV8M;
#[cfg(all(target_arch = "arm", not(armv8m_target)))]
const STORE_BASE: usize = abi::STORE_BASE_ARM;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
const STORE_BASE: usize = abi::STORE_BASE_RISCV;
// Host: no store region; probe() returns NoStore and callers fall back
// to the embedded image path.
#[cfg(not(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64")))]
const STORE_BASE: usize = 0;

/// Store validation / access failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreError {
    /// No store provisioned at this platform's store address.
    NoStore,
    /// Magic mismatch — region holds something else.
    BadMagic,
    /// Layout revision newer than this kernel understands.
    BadVersion,
    /// Store was provisioned for a different board.
    BoardMismatch,
    /// total_size implausible or directory inconsistent.
    BadSize,
    /// CRC32 over directory + images does not match the header.
    CrcMismatch,
    /// No entry with that name (or it is disabled).
    NotFound,
    /// The named module failed verification/start in the loader
    /// (see [`crate::modules::ModuleError`] for the underlying cause).
    LoadFailed,
}

/// Validated store view: header summary + zero-copy image access.
pub struct Store {
    data: &'static [u8],
    board_id: u16,
    entry_count: usize,
}

impl Store {
    /// Provisioned board id (as stored — may differ from `board::BOARD_ID`
    /// only when it is the unprovisioned wildcard).
    pub fn board_id(&self) -> u16 {
        self.board_id
    }

    /// Number of directory entries (enabled or not).
    pub fn entry_count(&self) -> usize {
        self.entry_count
    }

    fn entry(&self, idx: usize) -> StoreEntry {
        let off = abi::STORE_DIR_OFF + idx * core::mem::size_of::<StoreEntry>();
        // SAFETY: idx < entry_count <= STORE_MAX_ENTRIES, validated in probe;
        // the directory lies fully inside `data` (total_size check).
        unsafe { ptr::read_unaligned(self.data.as_ptr().add(off) as *const StoreEntry) }
    }

    /// Resolve a module image slice by name (enabled entries only).
    pub fn image(&self, name: &str) -> Result<&'static [u8], StoreError> {
        for idx in 0..self.entry_count {
            let e = self.entry(idx);
            if !e.enabled() || e.name_str() != name {
                continue;
            }
            let start = e.offset as usize;
            let size = e.size as usize;
            if start < abi::STORE_IMG_BASE || start + size > self.data.len()
            {
                return Err(StoreError::BadSize);
            }
            // SAFETY: bounds-checked above; the store region is firmware/
            // bootloader-owned and unmodified for the kernel lifetime, so
            // the 'static borrow holds by platform contract.
            return Ok(unsafe { core::slice::from_raw_parts(self.data.as_ptr().add(start), size) });
        }
        Err(StoreError::NotFound)
    }
}

// ---------------------------------------------------------------------------
// Probe + validation (shared by target probe and host tests)
// ---------------------------------------------------------------------------

/// Validate a store image byte slice. `expect_board` is the running board's
/// identity; `UNPROVISIONED` stores (0) are accepted as a wildcard.
fn parse_store(data: &[u8], expect_board: u16) -> Result<Store, StoreError> {
    const DIR_END: usize = abi::STORE_DIR_OFF + abi::STORE_MAX_ENTRIES * 24;
    if data.len() < DIR_END {
        return Err(StoreError::BadSize);
    }
    let header = StoreHeader {
        magic: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
        version: u16::from_le_bytes([data[4], data[5]]),
        board_id: u16::from_le_bytes([data[6], data[7]]),
        entry_count: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        total_size: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
        crc32: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
        reserved: u32::from_le_bytes([data[20], data[21], data[22], data[23]]),
    };
    if header.magic != abi::STORE_MAGIC {
        return Err(StoreError::BadMagic);
    }
    if header.version != abi::STORE_VERSION {
        return Err(StoreError::BadVersion);
    }
    if header.board_id != expect_board && header.board_id != abi::board_id::UNPROVISIONED {
        return Err(StoreError::BoardMismatch);
    }
    let total = header.total_size as usize;
    if total < DIR_END || total > data.len() {
        return Err(StoreError::BadSize);
    }
    if header.entry_count as usize > abi::STORE_MAX_ENTRIES {
        return Err(StoreError::BadSize);
    }
    if header.reserved != 0 {
        return Err(StoreError::BadSize);
    }
    if crate::kapi::crc32(&data[abi::STORE_DIR_OFF..total]) != header.crc32 {
        return Err(StoreError::CrcMismatch);
    }
    Ok(Store {
        // SAFETY: `data` borrows the store region, which is provisioned
        // once by the bootloader and never mutated for the kernel lifetime
        // ('static platform contract — same as the DTB view in dtb.rs).
        data: unsafe { core::slice::from_raw_parts(data.as_ptr(), total) },
        board_id: header.board_id,
        entry_count: header.entry_count as usize,
    })
}

/// Probe the store and load the named module by name (the Phase 4 flow:
/// flash store -> verify -> RAM slot -> start).
pub fn load(name: &str) -> Result<usize, StoreError> {
    let st = probe()?;
    let image = st.image(name)?;
    crate::modules::load_image(image).map_err(|_| StoreError::LoadFailed)
}

/// Probe this platform's store address and validate it.
pub fn probe() -> Result<Store, StoreError> {
    if STORE_BASE == 0 {
        return Err(StoreError::NoStore);
    }
    // SAFETY: STORE_BASE is the platform store address; the header probe
    // reads 32 bytes, and parse_store bounds every further access against
    // the header-declared total_size.
    let data =
        unsafe { core::slice::from_raw_parts(STORE_BASE as *const u8, abi::MODULE_SLOT_SIZE * 2) };
    parse_store(data, board::BOARD_ID)
}

// ---------------------------------------------------------------------------
// Host tests — synthetic store built byte-by-byte (same format build_store.py)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use abi::{ModuleDescriptor, StoreEntry, StoreHeader};

    /// Directory end offset (24-byte header + 8 entries x 24 bytes).
    const DIR_END: usize = abi::STORE_DIR_OFF + abi::STORE_MAX_ENTRIES * 24;

    fn entry_bytes(name: &str, offset: u32, size: u32, flags: u32) -> heapless::Vec<u8, 32> {
        let mut e = heapless::Vec::new();
        let mut name_buf = [0u8; 12];
        let n = name.len().min(12);
        name_buf[..n].copy_from_slice(&name.as_bytes()[..n]);
        e.extend_from_slice(&name_buf).unwrap();
        e.extend_from_slice(&offset.to_le_bytes()).unwrap();
        e.extend_from_slice(&size.to_le_bytes()).unwrap();
        e.extend_from_slice(&flags.to_le_bytes()).unwrap();
        e
    }

    /// zlib CRC32 (mirrors kapi::crc32; kept local so a regression in one
    /// implementation is caught by the other).
    fn zlib_crc(data: &[u8]) -> u32 {
        let mut state: u32 = 0xFFFF_FFFF;
        for &b in data {
            state ^= b as u32;
            for _ in 0..8 {
                let lsb = state & 1;
                state >>= 1;
                if lsb != 0 {
                    state ^= 0xEDB8_8320;
                }
            }
        }
        state ^ 0xFFFF_FFFF
    }

    /// Store with one enabled "hello" image at DIR_END.
    /// `board` selects the provisioned board id; `corrupt` breaks the CRC.
    /// NOTE: the returned Vec must outlive the `Store` views built from it.
    fn make_store(board: u16, corrupt: bool) -> heapless::Vec<u8, 1024> {
        // Module image (header + descriptor + body; CRC patched like the
        // real post-processor).
        let mut img = heapless::Vec::<u8, 256>::new();
        img.extend_from_slice(&abi::MODULE_HEADER_MAGIC.to_le_bytes()).unwrap();
        img.extend_from_slice(&abi::KAPI_ABI_VERSION.to_le_bytes()).unwrap();
        img.extend_from_slice(&0u16.to_le_bytes()).unwrap(); // arch
        img.extend_from_slice(&(32u32 + 40 + 22).to_le_bytes()).unwrap(); // size
        img.extend_from_slice(&0u32.to_le_bytes()).unwrap(); // reserved0
        img.extend_from_slice(&0u32.to_le_bytes()).unwrap(); // crc placeholder
        img.extend_from_slice(&[0u8; 12]).unwrap(); // reserved
        let desc = ModuleDescriptor {
            magic: abi::MODULE_DESC_MAGIC,
            abi_version: abi::KAPI_ABI_VERSION,
            arch: 0,
            name: *b"hello\0\0\0\0\0\0\0\0\0\0\0",
            init: None,
            deinit: None,
        };
        img.extend_from_slice(&desc.magic.to_le_bytes()).unwrap();
        img.extend_from_slice(&desc.abi_version.to_le_bytes()).unwrap();
        img.extend_from_slice(&desc.arch.to_le_bytes()).unwrap();
        img.extend_from_slice(&desc.name).unwrap();
        img.extend_from_slice(&0usize.to_le_bytes()).unwrap(); // init
        img.extend_from_slice(&0usize.to_le_bytes()).unwrap(); // deinit
        img.extend_from_slice(b"hello-module-fake-body").unwrap();
        let isize_ = img.len() as u32;
        img[8..12].copy_from_slice(&isize_.to_le_bytes());
        let icrc = zlib_crc(&img[32..isize_ as usize]);
        img[16..20].copy_from_slice(&icrc.to_le_bytes());

        // Directory (192 B, one enabled entry) + image.
        let mut payload = heapless::Vec::<u8, 512>::new();
        let e = entry_bytes("hello", DIR_END as u32, isize_, abi::STORE_FLAG_ENABLED);
        payload.extend_from_slice(&e).unwrap();
        payload.extend_from_slice(&[0u8; DIR_END - abi::STORE_DIR_OFF - 24]).unwrap();
        payload.extend_from_slice(&img).unwrap();

        // Header (CRC over directory + images).
        let total = (abi::STORE_DIR_OFF + payload.len()) as u32;
        let crc = zlib_crc(&payload);
        let mut out = heapless::Vec::new();
        out.extend_from_slice(&abi::STORE_MAGIC.to_le_bytes()).unwrap();
        out.extend_from_slice(&abi::STORE_VERSION.to_le_bytes()).unwrap();
        out.extend_from_slice(&board.to_le_bytes()).unwrap();
        out.extend_from_slice(&1u32.to_le_bytes()).unwrap();
        out.extend_from_slice(&total.to_le_bytes()).unwrap();
        if corrupt {
            out.extend_from_slice(&0xDEAD_BEEFu32.to_le_bytes()).unwrap();
        } else {
            out.extend_from_slice(&crc.to_le_bytes()).unwrap();
        }
        out.extend_from_slice(&0u32.to_le_bytes()).unwrap();
        out.extend_from_slice(&payload).unwrap();
        out
    }

    #[test]
    fn probe_parses_and_resolves_image_by_name() {
        // The Vec outlives the Store view for the whole test body.
        let store_bytes = make_store(abi::board_id::UNPROVISIONED, false);
        let s = parse_store(&store_bytes, crate::board::BOARD_ID)
            .expect("unprovisioned board id is a wildcard");
        assert_eq!(s.board_id(), abi::board_id::UNPROVISIONED);
        assert_eq!(s.entry_count(), 1);
        let img = s.image("hello").expect("entry resolves");
        assert_eq!(&img[0..4], &abi::MODULE_HEADER_MAGIC.to_le_bytes());
        assert!(s.image("missing").is_err());
    }

    #[test]
    fn board_mismatch_rejected() {
        let store_bytes = make_store(0x0B0B, false); // provisioned for another board
        match parse_store(&store_bytes, crate::board::BOARD_ID) {
            Err(StoreError::BoardMismatch) => {} // expected
            other => panic!("expected BoardMismatch, got {:?}", other.err()),
        }
    }

    #[test]
    fn exact_board_match_accepted() {
        let store_bytes = make_store(0x0002, false); // QEMU_VIRT_RV32IMAC
        let s = parse_store(&store_bytes, 0x0002).expect("exact board accepted");
        assert_eq!(s.board_id(), 0x0002);
    }

    #[test]
    fn corrupted_store_fails_crc() {
        let store_bytes = make_store(abi::board_id::UNPROVISIONED, true);
        match parse_store(&store_bytes, crate::board::BOARD_ID) {
            Err(StoreError::CrcMismatch) => {} // expected
            other => panic!("expected CrcMismatch, got {:?}", other.err()),
        }
    }
}