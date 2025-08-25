//! Architecture abstraction layer
//! This module provides a unified interface for architecture-specific functionality

use crate::config::ArchConfig;

#[cfg(target_arch = "arm")]
pub mod arm;
#[cfg(target_arch = "riscv32")]
pub mod riscv;

/// Initialize architecture-specific features
pub fn init_arch(arch_config: &ArchConfig) {
    #[cfg(target_arch = "arm")]
    {
        arm::early_init();
        arm::irq_init();
        arm::startup_init(arch_config);
    }
    
    #[cfg(target_arch = "riscv32")]
    {
        riscv::early_init();
        riscv::irq_init();
        riscv::startup_init(arch_config);
    }
}

/// Early debug output (available before full driver initialization)
pub fn early_println(msg: &str) {
    #[cfg(target_arch = "arm")]
    arm::early_println(msg);
    
    #[cfg(target_arch = "riscv32")]
    riscv::early_println(msg);
}

/// Architecture-specific printf for drivers (legacy compatibility)
pub fn arch_println(msg: &str) {
    // For now, just delegate to early_println
    early_println(msg);
}

/// Architecture-specific yield (wait for interrupt)
pub fn arch_yield() {
    #[cfg(target_arch = "arm")]
    {
        // ARM Cortex-M wait for interrupt
        cortex_m::asm::wfi();
    }
    
    #[cfg(target_arch = "riscv32")]
    riscv::arch_yield();
}

/// Architecture-specific shutdown
pub fn arch_shutdown() -> ! {
    #[cfg(target_arch = "arm")]
    {
        // ARM Cortex-M shutdown
        loop {
            arch_yield();
        }
    }
    
    #[cfg(target_arch = "riscv32")]
    riscv::arch_shutdown();
    
    #[cfg(not(any(target_arch = "arm", target_arch = "riscv32")))]
    {
        loop {}
    }
}
