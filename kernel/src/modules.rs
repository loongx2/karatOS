//! ============================================================================
//! MODULE : modules — flash module loader (extensible + retractable)
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   Implements the karatOS dynamic extensibility contract inside 64 kB:
//!     1. a module IMAGE (header + descriptor + code) sits in flash —
//!        today embedded via `.module_store` / include_bytes, Phase 4 turns
//!        this into a real flash store region;
//!     2. `load_image()` VERIFIES it (magic / arch / ABI / CRC32), COPIES it
//!        into a fixed RAM execution slot, and CALLS its init();
//!     3. the module registers itself through the KAPI table;
//!     4. `retract()` calls deinit() and WIPES the slot — the SRAM spent on
//!        the module is fully reclaimed. Extensibility is bounded by data
//!        SRAM, exactly as designed.
//!
//! WHY FIXED SLOTS INSTEAD OF A DYNAMIC LINKER
//!   Modules are linked against a fixed slot address and the fixed KAPI
//!   address. The loader therefore needs ZERO relocations — verification is
//!   a CRC, not a symbol walk. This is the single biggest size saving in the
//!   whole design.
//!
//! CONCURRENCY
//!   Load/retract/list run from the main loop only (same contract as the
//!   driver registry). The tick ISR never enters this module.
//!
//! MEMORY BUDGET
//!   One 8 kB RAM slot per arch + a tiny registry array in .bss; loader code
//!   ~1.5 kB ROM.
//! ============================================================================

use core::ptr;

pub use karatos_kapi as abi;

use abi::{ModuleDescriptor, ModuleHeader};
use crate::drivers;

/// Registry capacity: concurrently loaded modules.
pub const MAX_MODULES: usize = 4;

/// Loader failure reasons (returned to the demo loop for exact reporting).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleError {
    /// No embedded module image compiled in (build without the module).
    #[allow(dead_code)] // constructed only on offline-fallback builds
    NoImage,
    /// Image shorter than header + descriptor.
    TooSmall,
    /// Header magic mismatch — not a karatOS module.
    BadHeaderMagic,
    /// Descriptor magic mismatch — corrupt or truncated image.
    BadDescMagic,
    /// Built against a different KAPI ABI version.
    AbiMismatch,
    /// Built for a different CPU.
    WrongArch,
    /// Image larger than the RAM slot.
    TooBig,
    /// CRC32 over the image body does not match the header.
    CrcMismatch,
    /// All slots (or the registry) are in use.
    Full,
    /// The module's init() returned a non-zero status.
    InitFailed,
    /// Retract requested for an index that holds no module.
    NotFound,
}

/// One loaded module.
struct ModuleInstance {
    /// Descriptor address INSIDE the RAM slot (valid while loaded).
    desc: *const ModuleDescriptor,
    /// Bytes occupied in the slot (for wipe on retract).
    /// Only the target-only retract path reads it, hence the allow on host.
    #[cfg_attr(
        not(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64")),
        allow(dead_code)
    )]
    size: usize,
}

// Registry state — main-loop context only (see module docs).
struct Registry {
    entries: [Option<ModuleInstance>; MAX_MODULES],
}
static mut REGISTRY: Registry = Registry {
    entries: [const { None }; MAX_MODULES],
};

/// Number of currently loaded modules.
#[allow(dead_code)] // introspection surface for the Phase 5 shell/monitor
pub fn loaded_count() -> usize {
    // SAFETY: main-loop context only (module-level contract).
    unsafe {
        let reg = &*core::ptr::addr_of_mut!(REGISTRY);
        reg.entries.iter().filter(|e| e.is_some()).count()
    }
}

/// Print the registry through the console: `[idx] name` per loaded module.
pub fn list() {
    // SAFETY: main-loop context only.
    let reg = unsafe { &*core::ptr::addr_of_mut!(REGISTRY) };
    let mut n = 0;
    for (idx, entry) in reg.entries.iter().enumerate() {
        if let Some(inst) = entry {
            let name = unsafe { (*inst.desc).name_str() };
            let mut line = heapless::String::<32>::new();
            let _ = core::fmt::write(&mut line, format_args!("  [{}] {}\r\n", idx, name));
            drivers::console_print(line.as_str());
            n += 1;
        }
    }
    if n == 0 {
        drivers::console_print("  (no modules loaded)\r\n");
    }
}

/// Internal registration used by the KAPI `register_module` export.
/// SAFETY: `desc` must point into a verified slot image.
pub(crate) fn register_descriptor(
    desc: *const abi::ModuleDescriptor,
) -> Result<usize, ModuleError> {
    // SAFETY: single-threaded boot/loop context (module-level contract).
    let reg = unsafe { &mut *core::ptr::addr_of_mut!(REGISTRY) };
    let magic = unsafe { (*desc).magic };
    if magic != abi::MODULE_DESC_MAGIC {
        return Err(ModuleError::BadDescMagic);
    }
    for slot in reg.entries.iter_mut() {
        if slot.is_none() {
            *slot = Some(ModuleInstance { desc, size: 0 });
            return Ok(0);
        }
    }
    Err(ModuleError::Full)
}

// ---------------------------------------------------------------------------
// Load path
// ---------------------------------------------------------------------------

/// Per-target RAM slot description.
#[cfg(all(target_arch = "arm", armv8m_target))]
const SLOT_BASE: usize = abi::MODULE_SLOT_ARMV8M;
#[cfg(all(target_arch = "arm", not(armv8m_target)))]
const SLOT_BASE: usize = abi::MODULE_SLOT_ARM;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
const SLOT_BASE: usize = abi::MODULE_SLOT_RISCV;
#[cfg(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64"))]
const SLOT_SIZE: usize = abi::MODULE_SLOT_SIZE;
// Host: no real slot; loader logic is exercised by the unit tests below,
// which only touch the validation path (TooSmall fails before slot access).
#[cfg(not(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64")))]
const SLOT_BASE: usize = 0;
#[cfg(not(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64")))]
const SLOT_SIZE: usize = abi::MODULE_SLOT_SIZE;

/// This CPU's ELF machine id, for header validation.
const fn target_machine() -> u16 {
    #[cfg(target_arch = "arm")]
    {
        abi::EM_ARM
    }
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    {
        abi::EM_RISCV
    }
    #[cfg(not(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64")))]
    {
        0
    }
}

/// Load and start a module image from a byte slice (flash store today,
/// real flash region in Phase 4).
pub fn load_image(image: &[u8]) -> Result<usize, ModuleError> {
    // -- 1. header ----------------------------------------------------------
    if image.len() < size_of::<ModuleHeader>() + size_of::<ModuleDescriptor>() {
        return Err(ModuleError::TooSmall);
    }
    let header = header_of(image);
    if header.magic != abi::MODULE_HEADER_MAGIC {
        return Err(ModuleError::BadHeaderMagic);
    }
    if header.abi_version != abi::KAPI_ABI_VERSION {
        return Err(ModuleError::AbiMismatch);
    }
    if header.arch != target_machine() {
        return Err(ModuleError::WrongArch);
    }
    let size = header.image_size as usize;
    if size > SLOT_SIZE || size > image.len() {
        return Err(ModuleError::TooBig);
    }

    // -- 2. integrity: CRC32 over the image body (everything after header) --
    let body = &image[core::mem::size_of::<ModuleHeader>()..size];
    if crate::kapi::crc32(body) != header.crc32 {
        return Err(ModuleError::CrcMismatch);
    }

    // -- 3. claim a slot and copy the image into RAM ------------------------
    let idx = free_slot()?;
    let slot = SLOT_BASE as *mut u8;
    // SAFETY: slot is a dedicated 8 kB RAM region reserved by the platform
    // memory map; `size` was bounds-checked against SLOT_SIZE above.
    unsafe {
        ptr::copy_nonoverlapping(image.as_ptr(), slot, size);
    }

    // -- 4. validate the descriptor that is now in RAM ----------------------
    // SAFETY: offset 32 lies within the copied image (size check above).
    let desc = unsafe { slot.add(abi::MODULE_DESC_OFFSET) } as *const ModuleDescriptor;
    // SAFETY: descriptor lies fully inside the copied image (size check).
    let desc_ref = unsafe { &*desc };
    if desc_ref.magic != abi::MODULE_DESC_MAGIC
        || desc_ref.abi_version != abi::KAPI_ABI_VERSION
        || desc_ref.arch != target_machine()
    {
        return Err(ModuleError::BadDescMagic);
    }

    // -- 5. run the module's init -------------------------------------------
    // SAFETY: verified image in its linked slot; ABI fixed by karatos-kapi.
    let init = desc_ref.init;
    if let Some(init) = init {
        if unsafe { init() } != abi::status::OK {
            return Err(ModuleError::InitFailed);
        }
    }

    // -- 6. publish in the registry -----------------------------------------
    record(idx, desc, size);
    Ok(idx)
}

const fn size_of<T>() -> usize {
    core::mem::size_of::<T>()
}

fn header_of(image: &[u8]) -> ModuleHeader {
    // SAFETY: length checked by caller; header is `repr(C)` POD, so an
    // unaligned byte read is safe (offset 0 of the slice).
    unsafe { ptr::read_unaligned(image.as_ptr() as *const ModuleHeader) }
}

fn free_slot() -> Result<usize, ModuleError> {
    // SAFETY: main-loop context only.
    let reg = unsafe { &mut *core::ptr::addr_of_mut!(REGISTRY) };
    for (idx, slot) in reg.entries.iter().enumerate() {
        if slot.is_none() {
            return Ok(idx);
        }
    }
    Err(ModuleError::Full)
}

fn record(idx: usize, desc: *const ModuleDescriptor, size: usize) {
    // SAFETY: main-loop context only.
    let reg = unsafe { &mut *core::ptr::addr_of_mut!(REGISTRY) };
    reg.entries[idx] = Some(ModuleInstance { desc, size });
}

/// Retract a loaded module: run `deinit()`, then wipe its slot so neither
/// the code nor the descriptor can be reached again. SRAM is reclaimed.
pub fn retract(idx: usize) -> Result<(), ModuleError> {
    // SAFETY: main-loop context only.
    let reg = unsafe { &mut *core::ptr::addr_of_mut!(REGISTRY) };
    let Some(inst) = reg.entries[idx].take() else {
        return Err(ModuleError::NotFound);
    };

    // SAFETY: descriptor was validated at load; it stays in our slot because
    // only this function clears registry entries.
    let desc = unsafe { &*inst.desc };
    if let Some(deinit) = desc.deinit {
        unsafe { deinit() };
    }

    // Wipe the slot: zero the whole copied image (descriptor magic, fn
    // pointers, code), so a stray call cannot reach retracted code.
    // SAFETY: dedicated RAM region, owned by the loader while loaded.
    #[cfg(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64"))]
    unsafe {
        let base = SLOT_BASE as *mut u8;
        ptr::write_bytes(base, 0, inst.size);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Host unit tests — CRC + header/descriptor round-trip, no MMIO
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use abi::ModuleDescriptor;

    fn crc32_ref(data: &[u8]) -> u32 {
        crate::kapi::crc32(data)
    }

    #[test]
    fn crc32_known_vectors() {
        // Standard zlib CRC32 check values.
        assert_eq!(crc32_ref(b""), 0);
        assert_eq!(crc32_ref(b"123456789"), 0xCBF4_3926);
        assert_eq!(
            crc32_ref(b"The quick brown fox jumps over the lazy dog"),
            0x414F_A339
        );
    }

    /// Build a well-formed little-endian module image (header + descriptor +
    /// body) with size/CRC patched exactly like the build post-processor.
    fn make_image() -> heapless::Vec<u8, 512> {
        let mut img = heapless::Vec::<u8, 512>::new();
        let body: &[u8] = b"pretend module code!";
        let desc = ModuleDescriptor {
            magic: abi::MODULE_DESC_MAGIC,
            abi_version: abi::KAPI_ABI_VERSION,
            arch: 0, // arch-agnostic for the test
            name: *b"hello\0\0\0\0\0\0\0\0\0\0\0",
            init: None,
            deinit: None,
        };
        let image_size: u32 = 32 + 40 + body.len() as u32;
        // header (32 bytes)
        img.extend_from_slice(&abi::MODULE_HEADER_MAGIC.to_le_bytes()).unwrap();
        img.extend_from_slice(&abi::KAPI_ABI_VERSION.to_le_bytes()).unwrap();
        img.extend_from_slice(&0u16.to_le_bytes()).unwrap();
        img.extend_from_slice(&image_size.to_le_bytes()).unwrap();
        img.extend_from_slice(&0u32.to_le_bytes()).unwrap(); // reserved0
        img.extend_from_slice(&0u32.to_le_bytes()).unwrap(); // crc placeholder
        img.extend_from_slice(&[0u8; 12]).unwrap(); // reserved
        // descriptor (40 bytes: 4+2+2+16+8)
        img.extend_from_slice(&desc.magic.to_le_bytes()).unwrap();
        img.extend_from_slice(&desc.abi_version.to_le_bytes()).unwrap();
        img.extend_from_slice(&desc.arch.to_le_bytes()).unwrap();
        img.extend_from_slice(&desc.name).unwrap();
        img.extend_from_slice(&0usize.to_le_bytes()).unwrap(); // init
        img.extend_from_slice(&0usize.to_le_bytes()).unwrap(); // deinit
        // body
        img.extend_from_slice(body).unwrap();
        // patch size + crc like the build post-processor does
        img[8..12].copy_from_slice(&image_size.to_le_bytes());
        let crc = crc32_ref(&img[32..image_size as usize]);
        img[16..20].copy_from_slice(&crc.to_le_bytes());
        img
    }

    #[test]
    fn header_round_trip_and_validation() {
        let img = make_image();
        let header = header_of(&img);
        assert_eq!(header.magic, abi::MODULE_HEADER_MAGIC);
        assert_eq!(header.abi_version, abi::KAPI_ABI_VERSION);
        assert_eq!(
            crate::kapi::crc32(&img[32..header.image_size as usize]),
            header.crc32
        );
    }

    #[test]
    fn corrupted_body_fails_crc() {
        let mut img = make_image();
        let last = img.len() - 1;
        img[last] ^= 0xFF; // flip one bit in the body
        let header = header_of(&img);
        assert_ne!(
            crate::kapi::crc32(&img[32..header.image_size as usize]),
            header.crc32
        );
    }

    #[test]
    fn descriptor_name_round_trip() {
        let img = make_image();
        let desc = unsafe { &*(img.as_ptr().add(32) as *const ModuleDescriptor) };
        assert_eq!(desc.name_str(), "hello");
    }

    #[test]
    fn too_small_image_rejected() {
        assert_eq!(load_image(&[0u8; 8]), Err(ModuleError::TooSmall));
    }
}