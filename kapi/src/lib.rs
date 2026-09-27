//! ============================================================================
//! karatos-kapi — the ONE source of truth for the kernel<->module ABI
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   Everything a loadable karatOS module and the kernel must agree on:
//!     * `ModuleHeader`     — flash image header (magic/abi/arch/size/crc32)
//!     * `ModuleDescriptor` — what a module exposes at image offset 32
//!     * `KapiTable`        — the kernel's export table at a FIXED address
//!     * fixed addresses    — per-arch KAPI location + RAM module slot
//!
//! STABILITY CONTRACT
//!   This crate is deliberately tiny and dependency-free. Breaking any
//!   `#[repr(C)]` layout, constant, or fn signature requires bumping
//!   `KAPI_ABI_VERSION` — modules compiled against a different version are
//!   rejected by the loader before any code runs.
//!
//! CALLING MODEL
//!   Modules are linked for a FIXED RAM slot address (no runtime relocation),
//!   so: (a) their code may call the kernel ONLY through the `KapiTable`
//!   function pointers read from `KAPI_ADDR`, and (b) the kernel may call the
//!   module's init/deinit directly through the descriptor. All cross-boundary
//!   functions are `extern "C"` with C-safe argument types.
//!
//! MEMORY BUDGET
//!   Types and constants only — compiles to nothing on either side.
//! ============================================================================
#![no_std]

// ---------------------------------------------------------------------------
// ABI version + fixed addresses
// ---------------------------------------------------------------------------

/// ABI contract revision. Bump on ANY layout/semantic change below.
pub const KAPI_ABI_VERSION: u16 = 1;

/// ELF machine ids used to tag module images (fields are u16).
pub const EM_ARM: u16 = 40;
pub const EM_RISCV: u16 = 243;

/// Module image magic ("KMOD" little-endian) at image offset 0.
pub const MODULE_HEADER_MAGIC: u32 = 0x4B4D_4F44;
/// Module descriptor magic ("KMDD") at image offset 32.
pub const MODULE_DESC_MAGIC: u32 = 0x4B4D_4444;
/// Kernel KAPI table magic ("KAPI").
pub const KAPI_TABLE_MAGIC: u32 = 0x4150_414B;

/// Offset of the `ModuleDescriptor` inside a module image.
pub const MODULE_DESC_OFFSET: usize = 32;

/// Fixed KAPI table address — ARM (LM3S6965EVB, 256K flash @ 0x0000_0000).
pub const KAPI_ADDR_ARM: usize = 0x0001_F000;
/// Fixed KAPI table address — RISC-V (QEMU `virt`, RAM @ 0x8000_0000).
pub const KAPI_ADDR_RISCV: usize = 0x8001_0000;

/// RAM execution slot for loaded modules — ARMv7-M (64K SRAM @ 0x2000_0000).
pub const MODULE_SLOT_ARM: usize = 0x2000_4000;
/// Fixed KAPI table address — ARMv8-M (MPS3-AN547 flash @ 0x1000_0000).
pub const KAPI_ADDR_ARMV8M: usize = 0x1001_F000;
/// RAM execution slot — ARMv8-M (2M SRAM @ 0x3000_0000).
pub const MODULE_SLOT_ARMV8M: usize = 0x3000_4000;
/// RAM execution slot for loaded modules — RISC-V (QEMU `virt`).
pub const MODULE_SLOT_RISCV: usize = 0x8000_8000;

/// Maximum module image size that fits a slot (also the slot length).
pub const MODULE_SLOT_SIZE: usize = 8 * 1024;

// ---------------------------------------------------------------------------
// Module STORE (Phase 4) — the flash region modules are provisioned into
// ---------------------------------------------------------------------------

/// Store magic ("KSTO" little-endian) at the store base address.
pub const STORE_MAGIC: u32 = 0x4B53_544F;
/// Store layout revision.
pub const STORE_VERSION: u16 = 1;
/// Directory capacity (max provisioned modules per store image).
pub const STORE_MAX_ENTRIES: usize = 8;
/// Store flags: entry is enabled (else the loader skips it).
pub const STORE_FLAG_ENABLED: u32 = 1 << 0;

/// Offset of the directory (the header is 24 bytes: 4+2+2+4+4+4+4).
pub const STORE_DIR_OFF: usize = 24;
/// Offset of the first module image (directory + [`STORE_MAX_ENTRIES`]).
pub const STORE_IMG_BASE: usize = STORE_DIR_OFF + STORE_MAX_ENTRIES * 24;

/// OTP-equivalent board identities. The kernel refuses a store provisioned
/// for a different board (unless the store is unprovisioned / wildcard 0).
pub mod board_id {
    /// LM3S6965EVB (Cortex-M3, QEMU `-M lm3s6965evb`).
    pub const LM3S6965: u16 = 0x0001;
    /// QEMU `virt`, RV32IMAC.
    pub const QEMU_VIRT_RV32IMAC: u16 = 0x0002;
    /// QEMU `virt`, RV32IMC (ESP32-C3 class).
    pub const QEMU_VIRT_RV32IMC: u16 = 0x0003;
    /// QEMU `virt`, RV64GC.
    pub const QEMU_VIRT_RV64: u16 = 0x0004;
    /// MPS3-AN547 (Cortex-M33/M55).
    pub const MPS3_AN547: u16 = 0x0005;
    /// Store provisioned without a board identity (wildcard: always accepted).
    pub const UNPROVISIONED: u16 = 0x0000;
}

/// Per-target store base (compile-time selection by `target_arch`;
/// ARMv8-M uses `KAPI_ADDR_ARMV8M` geography — see `store_base_for`).
pub const STORE_BASE_ARM: usize = 0x0002_2000;
pub const STORE_BASE_ARMV8M: usize = 0x1002_1000;
/// RISC-V: the store lives at a fixed RAM address that the bootloader/QEMU
/// loader provisions pre-boot (Phase 4 model). Phase 5 maps it to pflash.
pub const STORE_BASE_RISCV: usize = 0x8300_0000;

// ---------------------------------------------------------------------------
// Store on-flash layout — header at store base, directory, then images
// ---------------------------------------------------------------------------
/// Store header — the first [`STORE_DIR_OFF`] bytes of the store region.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StoreHeader {
    /// [`STORE_MAGIC`]
    pub magic: u32,
    /// [`STORE_VERSION`]
    pub version: u16,
    /// Provisioned board identity ([`board_id`] constants; 0 = wildcard)
    pub board_id: u16,
    /// Number of valid directory entries (<= [`STORE_MAX_ENTRIES`])
    pub entry_count: u32,
    /// Total store image bytes (header + directory + all module images)
    pub total_size: u32,
    /// CRC32(IEEE) over `store[32..total_size]` (directory + all images)
    pub crc32: u32,
    /// Reserved, must be zero
    pub reserved: u32,
}

/// One directory entry: locates a module image inside the store.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StoreEntry {
    /// NUL-padded module name (lookup key)
    pub name: [u8; 12],
    /// Image offset from the store base (4-byte aligned)
    pub offset: u32,
    /// Image size in bytes (header + descriptor + code)
    pub size: u32,
    /// [`STORE_FLAG_ENABLED`] bit; others reserved (zero)
    pub flags: u32,
}

impl StoreEntry {
    /// NUL-padded name as `&str`.
    pub fn name_str(&self) -> &str {
        let len = self.name.iter().position(|&b| b == 0).unwrap_or(12);
        core::str::from_utf8(&self.name[..len]).unwrap_or("?")
    }

    /// Whether this entry is marked enabled.
    pub fn enabled(&self) -> bool {
        self.flags & STORE_FLAG_ENABLED != 0
    }
}

/// Per-target fixed addresses (compile-time selection by `target_arch`).
pub const fn kapi_addr() -> usize {
    #[cfg(target_arch = "arm")]
    {
        KAPI_ADDR_ARM
    }
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    {
        KAPI_ADDR_RISCV
    }
    #[cfg(not(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64")))]
    {
        0
    }
}

/// Per-target module RAM slot base.
pub const fn module_slot_base() -> usize {
    #[cfg(target_arch = "arm")]
    {
        MODULE_SLOT_ARM
    }
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    {
        MODULE_SLOT_RISCV
    }
    #[cfg(not(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64")))]
    {
        0
    }
}

// ---------------------------------------------------------------------------
// Module image header — bytes [0..32] of the flash image
// ---------------------------------------------------------------------------
/// Fixed-layout image header. The loader validates this BEFORE touching any
/// module code:
///   * `magic`    == [`MODULE_HEADER_MAGIC`]
///   * `abi_version` == [`KAPI_ABI_VERSION`]
///   * `arch`     == this kernel's ELF machine id
///   * `image_size` <= [`MODULE_SLOT_SIZE`]
///   * `crc32`    == CRC32(IEEE) over `image[32..image_size]`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ModuleHeader {
    /// [`MODULE_HEADER_MAGIC`]
    pub magic: u32,
    /// [`KAPI_ABI_VERSION`] this module was built against
    pub abi_version: u16,
    /// ELF machine id of the intended CPU
    pub arch: u16,
    /// Total image bytes, header included
    pub image_size: u32,
    /// Reserved for future use (entry offset / signature), must be zero
    pub reserved0: u32,
    /// CRC32(IEEE, zlib-compatible) over `image[32..image_size]`
    pub crc32: u32,
    /// Reserved, must be zero
    pub reserved: [u32; 3],
}

/// Status codes returned across the ABI boundary.
pub mod status {
    /// Operation completed successfully.
    pub const OK: i32 = 0;
    /// The kernel rejected the call (bad magic/version/registry full).
    pub const REJECTED: i32 = -1;
}

// ---------------------------------------------------------------------------
// Module descriptor — bytes [32..] of the image, first thing the kernel calls
// ---------------------------------------------------------------------------
/// What every karatOS module exports at image offset
/// [`MODULE_DESC_OFFSET`]. Function pointers are absolute addresses valid
/// after the image is placed at its linked slot address.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ModuleDescriptor {
    /// [`MODULE_DESC_MAGIC`]
    pub magic: u32,
    /// [`KAPI_ABI_VERSION`]
    pub abi_version: u16,
    /// ELF machine id (redundant with header; belt and braces)
    pub arch: u16,
    /// NUL-padded module name (registry key)
    pub name: [u8; 16],
    /// Module setup. Return [`status::OK`] on success.
    pub init: Option<unsafe extern "C" fn() -> i32>,
    /// Module teardown (retract). Must leave hardware as found.
    pub deinit: Option<unsafe extern "C" fn()>,
}

impl ModuleDescriptor {
    /// NUL-padded name as `&str` (stops at first NUL).
    pub fn name_str(&self) -> &str {
        let len = self.name.iter().position(|&b| b == 0).unwrap_or(16);
        match core::str::from_utf8(&self.name[..len]) {
            Ok(s) => s,
            Err(_) => "?",
        }
    }
}

// ---------------------------------------------------------------------------
// KAPI table — the kernel's export surface, read by modules at `kapi_addr()`
// ---------------------------------------------------------------------------
/// Kernel function export table. Placed in the kernel's `.kapi` section at
/// the per-arch fixed address; modules dereference `kapi_addr()` as this
/// type. Fields are APPEND-ONLY across versions (old modules keep working).
#[repr(C)]
pub struct KapiTable {
    /// [`KAPI_TABLE_MAGIC`]
    pub magic: u32,
    /// [`KAPI_ABI_VERSION`]
    pub version: u32,
    /// Blocking console write (registered UART driver).
    pub console_write: unsafe extern "C" fn(ptr: *const u8, len: usize),
    /// Milliseconds since boot.
    pub millis: unsafe extern "C" fn() -> u32,
    /// CRC32(IEEE) over `len` bytes — same algorithm the loader uses.
    pub crc32: unsafe extern "C" fn(ptr: *const u8, len: usize) -> u32,
    /// Register a module descriptor with the kernel registry.
    /// Returns [`status::OK`] or [`status::REJECTED`].
    pub register_module: unsafe extern "C" fn(desc: *const ModuleDescriptor) -> i32,
}