//! RISC-V specific functionality and hardware abstraction

use crate::arch::{ArchInit, MemoryLayout};

/// RISC-V architecture implementation
pub struct RiscvArch;

impl ArchInit for RiscvArch {
    fn init() {
        // Initialize RISC-V specific features
        Self::irq_init();
        Self::setup_memory_protection();
    }
    
    fn irq_init() {
        // Initialize interrupts for RISC-V
        // For now, just enable basic interrupt handling
    }
    
    fn setup_memory_protection() {
        // Set up PMP if available
        // For now, basic setup
    }
}

/// RISC-V memory layout implementation
pub struct RiscvMemoryLayout;

impl MemoryLayout for RiscvMemoryLayout {
    fn ram_start() -> usize {
        0x80000000 // Standard RISC-V RAM start
    }
    
    fn ram_size() -> usize {
        128 * 1024 // 128KB RAM for virt machine
    }
    
    fn flash_start() -> usize {
        0x20000000 // Flash start
    }
    
    fn flash_size() -> usize {
        512 * 1024 // 512KB Flash
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

/// Early debug output for RISC-V
pub fn early_println(msg: &str) {
    // Use simple UART output for RISC-V
    let uart_base = 0x10000000; // QEMU virt machine UART
    unsafe {
        for byte in msg.bytes() {
            core::ptr::write_volatile(uart_base as *mut u8, byte);
        }
        // Add newline
        core::ptr::write_volatile(uart_base as *mut u8, b'\n');
    }
}
