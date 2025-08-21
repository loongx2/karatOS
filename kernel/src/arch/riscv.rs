//! RISC-V Architecture Implementation
//! RISC-V-specific functionality and hardware abstraction

use core::arch::asm;

#[no_mangle]
pub extern "C" fn arch_init() {
    // RISC-V-specific initialization
    unsafe {
        // Set machine trap vector (simplified)
        asm!("csrw mtvec, zero");
        
        // Enable machine interrupts if needed
        // asm!("csrsi mstatus, 0x8");
    }
}

#[no_mangle]
pub extern "C" fn context_switch() {
    // RISC-V context switch implementation
    // Save/restore CPU state using RISC-V registers
}

#[inline(always)]
pub fn disable_interrupts() {
    unsafe {
        asm!("csrci mstatus, 8"); // Clear MIE bit (bit 3)
    }
}

#[inline(always)]
pub fn enable_interrupts() {
    unsafe {
        asm!("csrsi mstatus, 8"); // Set MIE bit (bit 3)
    }
}

pub fn arch_println(s: &str) {
    // RISC-V debug output - for now, this is a stub
    // In a real implementation, this might use a debug UART or SBI
    // For QEMU, we could implement SBI console output
    
    // Simple implementation: do nothing for now
    // The actual UART output will be handled by the driver layer
    let _ = s; // Suppress unused variable warning
}

#[inline(always)]
pub fn arch_yield() {
    unsafe {
        asm!("wfi"); // Wait for interrupt
    }
}

#[inline(always)]
pub fn arch_shutdown() -> ! {
    // RISC-V shutdown - could use SBI shutdown call
    unsafe {
        // Try to use QEMU's test device to exit
        let test_device = 0x100000 as *mut u32;
        core::ptr::write_volatile(test_device, 0x5555); // QEMU exit success
    }
    
    // Fallback: infinite loop
    loop {
        arch_yield();
    }
}
