//! riscv-rt runtime configuration hooks and symbols
//! Provides required symbols to satisfy riscv-rt link and boot expectations.

#![cfg(target_arch = "riscv32")]

// riscv-rt expects these weak symbols; we provide simple defaults for single-hart bring-up.

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

// Optional hook to set up interrupts before entering Rust main. Do nothing for now.
#[no_mangle]
pub extern "C" fn _setup_interrupts() {}

// Optional pre-init hook called very early. Do nothing.
#[no_mangle]
pub extern "C" fn __pre_init() {}

// Data section boundaries (will be set by linker)
#[no_mangle]
pub static mut _sdata: usize = 0;
#[no_mangle]
pub static mut _edata: usize = 0;
#[no_mangle]
pub static mut _sidata: usize = 0;

// BSS section boundaries (will be set by linker)
#[no_mangle]
pub static mut _sbss: usize = 0;
#[no_mangle]
pub static mut _ebss: usize = 0;
