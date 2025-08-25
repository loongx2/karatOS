//! ARM Cortex-M specific functionality and hardware abstraction

use crate::arch::{ArchInit, MemoryLayout};
use cortex_m_semihosting::hprintln;

/// ARM architecture implementation
pub struct ArmArch;

impl ArchInit for ArmArch {
    fn init() {
        // Initialize ARM-specific features
        Self::irq_init();
        Self::setup_memory_protection();
    }
    
    fn irq_init() {
        // Initialize interrupts for ARM
        // For now, just enable basic interrupt handling
    }
    
    fn setup_memory_protection() {
        // Set up MPU if available
        // For now, basic setup
    }
}

/// ARM memory layout implementation
pub struct ArmMemoryLayout;

impl MemoryLayout for ArmMemoryLayout {
    fn ram_start() -> usize {
        0x20000000 // Standard ARM Cortex-M RAM start
    }
    
    fn ram_size() -> usize {
        64 * 1024 // 64KB RAM for LM3S6965
    }
    
    fn flash_start() -> usize {
        0x00000000 // Flash start
    }
    
    fn flash_size() -> usize {
        256 * 1024 // 256KB Flash for LM3S6965
    }
    
    fn stack_top() -> usize {
        Self::ram_start() + Self::ram_size()
    }
    
    fn heap_start() -> usize {
        Self::ram_start() + (Self::ram_size() / 2) // Middle of RAM
    }
    
    fn heap_size() -> usize {
        Self::ram_size() / 4 // Quarter of RAM for heap
    }
}

/// Early debug output for ARM
pub fn early_println(msg: &str) {
    #[cfg(feature = "arm")]
    {
        // Use semihosting for early debug output
        let _ = hprintln!("{}", msg);
    }
}
