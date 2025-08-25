//! Kernel core module
//! Architecture-agnostic kernel initialization and management

use crate::arch::Architecture;
use crate::drivers;

/// Initialize the kernel for the current architecture
pub fn init() {
    // Initialize architecture-specific components
    Architecture::init();
    
    // Initialize drivers
    drivers::uart::init();
    
    // Print boot message
    drivers::uart::print("karatOS kernel initialized\n");
}

/// Main kernel loop
pub fn run() -> ! {
    drivers::uart::print("Kernel running...\n");
    
    loop {
        // Kernel main loop - for now just halt
        #[cfg(target_arch = "arm")]
        unsafe { core::arch::asm!("wfi") };
        
        #[cfg(target_arch = "riscv32")]
        unsafe { core::arch::asm!("wfi") };
    }
}
