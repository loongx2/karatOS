//! ============================================================================
//! MODULE : riscv_rt_config — riscv-rt runtime hooks (symbols riscv-rt expects)
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   Provides/overrides the weak default symbols riscv-rt links against:
//!   single-hart parking, per-hart stack sizing, and — critically —
//!   `_setup_interrupts`, which installs the trap vector.
//!
//! WHY _setup_interrupts MATTERS
//!   riscv-rt 0.12 only defines `default_setup_interrupts` (a differently
//!   named symbol); the `_setup_interrupts` call in `_start_rust` is an
//!   extern that WE must provide. An empty body would leave `mtvec` at its
//!   reset value (0), so the first trap (our CLINT tick!) would jump to
//!   address 0 and brick the machine. This implementation mirrors
//!   riscv-rt's default: mtvec = &_start_trap, direct mode.
//!
//! SYMBOL OWNERSHIP
//!   `_sdata/_edata/_sidata/_sbss/_ebss` are NOT defined here — they are
//!   linker-script symbols (build/templates/memory-riscv*.x) that the
//!   riscv-rt .data-copy / .bss-zero loops reference. Defining them as Rust
//!   statics would hijack the addresses and silently break RAM init.
//!
//! MEMORY BUDGET
//!   Two usize constants + three tiny fns; all resolved at link time.
//! ============================================================================

#![cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]

// Maximum hart id supported (single hart: 0)
#[no_mangle]
pub static _max_hart_id: usize = 0;

// Per-hart stack size (small but sufficient for early boot)
#[no_mangle]
pub static _hart_stack_size: usize = 4096;

// Multi-processor hook. Return true on primary hart only so others park.
#[no_mangle]
pub extern "C" fn _mp_hook(hart_id: usize) -> bool {
    // Only hart 0 continues
    hart_id == 0
}

// Optional pre-init hook called very early. Do nothing.
#[no_mangle]
pub extern "C" fn __pre_init() {}

/// Install the trap vector: `mtvec = &_start_trap` (direct mode).
///
/// This mirrors riscv-rt's `default_setup_interrupts`. Without it the CPU
/// would trap through an unset `mtvec` the first time the CLINT tick fires.
#[no_mangle]
pub extern "C" fn _setup_interrupts() {
    // riscv-rt's assembly trap entry point (saves/restores the frame).
    extern "C" {
        fn _start_trap();
    }
    // SAFETY: CSR write of a valid code address, direct mode — exactly what
    // riscv-rt's default implementation does.
    unsafe {
        riscv::register::mtvec::write(
            _start_trap as usize,
            riscv::register::mtvec::TrapMode::Direct,
        );
    }
}
