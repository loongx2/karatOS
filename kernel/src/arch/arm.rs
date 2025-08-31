//! ARM Cortex-M specific functionality and hardware abstraction

use crate::arch::{ArchInit, MemoryLayout};

// Exception handlers for ARM Cortex-M
#[cfg(target_arch = "arm")]
use cortex_m_rt::{exception};

/// Pre-init function called before main memory initialization
#[no_mangle]
pub unsafe extern "C" fn __pre_init() {
    // Nothing to do for basic setup
}

/// Default handler for unhandled interrupts
#[no_mangle]
pub unsafe extern "C" fn DefaultHandler() {
    loop {
        cortex_m::asm::wfi();
    }
}

// Exception handlers - cortex-m-rt requires these to be defined
#[exception]
unsafe fn NonMaskableInt() {
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn MemoryManagement() {
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn BusFault() {
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn UsageFault() {
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn SVCall() {
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn DebugMonitor() {
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn PendSV() {
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn SysTick() {
    loop {
        cortex_m::asm::wfi();
    }
}

// Hard fault handler
#[exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    // Print fault information via semihosting for debugging
    use cortex_m_semihosting::hprintln;
    let _ = hprintln!("Hard Fault at 0x{:x}", ef.pc());
    let _ = hprintln!("R0: 0x{:x}, R1: 0x{:x}, R2: 0x{:x}, R3: 0x{:x}", 
                     ef.r0(), ef.r1(), ef.r2(), ef.r3());
    
    loop {
        // Infinite loop on hard fault
        cortex_m::asm::wfi();
    }
}

/// ARM architecture implementation
pub struct ArmArch;

impl ArchInit for ArmArch {
    fn init() {
        // Initialize ARM-specific features
        ArmArch::init_uart();
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

impl ArmArch {
    fn init_uart() {
        // LM3S6965EVB UART0 initialization
        const RCGC1: usize = 0x400FE104; // Run mode clock gating control register 1
        const UART0_BASE: usize = 0x4000C000;
        const UARTIBRD: usize = UART0_BASE + 0x024; // Integer baud rate divisor
        const UARTFBRD: usize = UART0_BASE + 0x028; // Fractional baud rate divisor
        const UARTLCRH: usize = UART0_BASE + 0x02C; // Line control register
        const UARTCTL: usize = UART0_BASE + 0x030; // Control register
        
        unsafe {
            // Enable UART0 clock
            let rcgc1 = core::ptr::read_volatile(RCGC1 as *const u32);
            core::ptr::write_volatile(RCGC1 as *mut u32, rcgc1 | (1 << 0));
            
            // Configure UART for 115200 baud rate (assuming 16MHz system clock)
            // IBRD = 16MHz / (16 * 115200) = 8.6805 -> 8
            // FBRD = (0.6805 * 64) + 0.5 = 43.5 -> 44
            core::ptr::write_volatile(UARTIBRD as *mut u32, 8);
            core::ptr::write_volatile(UARTFBRD as *mut u32, 44);
            
            // Configure line control: 8 bits, no parity, 1 stop bit
            core::ptr::write_volatile(UARTLCRH as *mut u32, 0x60);
            
            // Enable UART, TX, RX
            core::ptr::write_volatile(UARTCTL as *mut u32, 0x301);
        }
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

/// Interrupt control functions for ARM Cortex-M
pub fn disable_interrupts() {
    unsafe {
        core::arch::asm!("cpsid i", options(nomem, nostack));
    }
}

pub fn enable_interrupts() {
    unsafe {
        core::arch::asm!("cpsie i", options(nomem, nostack));
    }
}

/// Early debug output for ARM
pub fn early_println(msg: &str) {
    // LM3S6965EVB UART0 at 0x4000C000
    const UART_BASE: usize = 0x4000C000;
    const UARTDR: usize = UART_BASE + 0x000; // Data register

    unsafe {
        for byte in msg.bytes() {
            // Write byte directly to UART data register
            // QEMU should handle the UART configuration
            core::ptr::write_volatile(UARTDR as *mut u32, byte as u32);
        }
        // Add newline
        core::ptr::write_volatile(UARTDR as *mut u32, b'\n' as u32);
    }
}
