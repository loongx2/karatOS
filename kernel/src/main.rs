//! karatOS - Multi-architecture RTOS kernel
//! Unified entry point for ARM and RISC-V targets

#![no_std]
#![no_main]

// Import modules directly since this is the main binary
mod arch;
mod config;
mod drivers;
mod kernel;
mod memory;

/// Main entry point for the kernel
/// This function is called by the architecture-specific boot code
#[no_mangle]
pub fn kernel_main() -> ! {
    // Initialize and run the kernel
    kernel::init();
    kernel::run()
}

// Architecture-specific entry points

/// ARM-specific entry point
#[cfg(target_arch = "arm")]
#[cortex_m_rt::entry]
fn main() -> ! {
    kernel_main()
}

/// RISC-V specific entry point  
#[cfg(target_arch = "riscv32")]
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    kernel_main()
}
