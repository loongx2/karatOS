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

/// RISC-V specific memory layout implementation
#[allow(dead_code)]
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

/// Interrupt control functions for RISC-V
pub fn disable_interrupts() {
    unsafe {
        riscv::register::mstatus::clear_mie();
    }
}

pub fn enable_interrupts() {
    unsafe {
        riscv::register::mstatus::set_mie();
    }
}

/// Early debug output for RISC-V
pub fn early_println(msg: &str) {
    // QEMU virt provides NS16550A UART at 0x1000_0000
    const UART_BASE: usize = 0x1000_0000;
    const THR: usize = UART_BASE + 0; // Transmit holding register
    const LSR: usize = UART_BASE + 5; // Line status register
    const LSR_THRE: u8 = 0x20; // Transmit holding register empty bit
    
    unsafe {
        for byte in msg.bytes() {
            // Wait for UART to be ready to transmit
            while (core::ptr::read_volatile(LSR as *const u8) & LSR_THRE) == 0 {
                // Busy wait - UART not ready
            }
            // Write byte to transmit holding register
            core::ptr::write_volatile(THR as *mut u8, byte);
        }
        // Add newline
        while (core::ptr::read_volatile(LSR as *const u8) & LSR_THRE) == 0 {
            // Busy wait - UART not ready
        }
        core::ptr::write_volatile(THR as *mut u8, b'\n');
    }
}

/// Yield CPU to other tasks (cooperative multitasking)
#[allow(dead_code)]
pub fn yield_cpu() {
    unsafe {
        // RISC-V wait for interrupt instruction
        core::arch::asm!("wfi", options(nomem, nostack));
    }
}

/// Shutdown system
#[allow(dead_code)]
pub fn shutdown() -> ! {
    // Disable interrupts and halt
    unsafe {
        core::arch::asm!("csrci mstatus, 8", options(nomem, nostack));
    }
    
    loop {
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack));
        }
    }
}
