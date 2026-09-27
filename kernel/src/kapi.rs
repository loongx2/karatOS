//! ============================================================================
//! MODULE : kapi — kernel side of the kernel<->module ABI (`.kapi` section)
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   Publishes the [`KapiTable`] — the ONLY surface loadable modules may
//!   call — as a `#[repr(C)]` static in the `.kapi` link section. The linker
//!   templates pin `.kapi` to a per-arch FIXED address (`kapi::kapi_addr()`),
//!   so modules bind to it at their link time: no relocation, no dynamic
//!   loader, a few hundred bytes of ROM. This is what makes extensibility
//!   affordable inside 64 kB.
//!
//! ROLE IN BOOT FLOW
//!   Static — live from image start. Every function behind it is safe to
//!   call once the driver registry is up (post kernel::init step 3).
//!
//! RULES FOR THE EXPORT SURFACE
//!   * `extern "C"`, C-safe argument types only (`*const u8` + len, u32...)
//!   * functions must never panic into a module (abort would kill the kernel)
//!   * additions are APPEND-ONLY; the table is versioned
//!
//! MEMORY BUDGET
//!   One struct of 6 words + thin wrappers, ~150 B ROM.
//! ============================================================================

pub use karatos_kapi as abi;

use crate::drivers;
use crate::modules;

/// The kernel export table — pinned to `.kapi` at the fixed per-arch address.
#[link_section = ".kapi"]
#[no_mangle]
pub static KARATOS_KAPI: abi::KapiTable = abi::KapiTable {
    magic: abi::KAPI_TABLE_MAGIC,
    version: abi::KAPI_ABI_VERSION as u32,
    console_write: kapi_console_write,
    millis: kapi_millis,
    crc32: kapi_crc32,
    register_module: kapi_register_module,
};

/// Blocking console write through the driver registry.
unsafe extern "C" fn kapi_console_write(ptr: *const u8, len: usize) {
    // SAFETY: module contract — ptr points to `len` readable bytes in memory
    // the module owns (its slot or kernel-shared rodata).
    let bytes = unsafe { core::slice::from_raw_parts(ptr, len) };
    match core::str::from_utf8(bytes) {
        Ok(s) => drivers::console_print(s),
        // Non-UTF8 payload: emit raw bytes through the first driver we find.
        Err(_) => drivers::console_print("[kapi:binary]"),
    }
}

/// Milliseconds since boot.
unsafe extern "C" fn kapi_millis() -> u32 {
    crate::arch::millis_since_boot()
}

/// CRC32(IEEE, zlib-compatible) — identical algorithm to the loader's check.
unsafe extern "C" fn kapi_crc32(ptr: *const u8, len: usize) -> u32 {
    // SAFETY: module contract — readable range.
    let bytes = unsafe { core::slice::from_raw_parts(ptr, len) };
    crc32(bytes)
}

/// Register a module descriptor (validates before accepting).
unsafe extern "C" fn kapi_register_module(desc: *const abi::ModuleDescriptor) -> i32 {
    match modules::register_descriptor(desc) {
        Ok(_) => abi::status::OK,
        Err(_) => abi::status::REJECTED,
    }
}

/// CRC32 (IEEE 802.3, reflected poly 0xEDB88320 — zlib-compatible).
/// Bitwise implementation: no 256-entry table, ~40 bytes of ROM.
#[inline]
pub fn crc32(data: &[u8]) -> u32 {
    crc32_update(0xFFFF_FFFF, data) ^ 0xFFFF_FFFF
}

/// Streaming CRC32 core: feed chunks with the running state (init 0xFFFF_FFFF),
/// final XOR by 0xFFFF_FFFF when done. Used by the loader to hash the module
/// body without building a temporary slice.
#[inline]
pub fn crc32_update(mut state: u32, data: &[u8]) -> u32 {
    for &byte in data {
        state ^= byte as u32;
        for _ in 0..8 {
            let lsb = state & 1;
            state >>= 1;
            if lsb != 0 {
                state ^= 0xEDB8_8320;
            }
        }
    }
    state
}