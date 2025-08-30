//! karatOS - Multi-architecture RTOS kernel
//! Unified entry point for ARM and RISC-V targets

#![no_std]
#![no_main]

// Include modules directly since this is the main binary
mod arch;
mod config;
mod drivers;
mod kernel;
mod memory;
#[cfg(target_arch = "riscv32")]
mod riscv_rt_config;

// External crates - conditionally imported based on target architecture
#[cfg(target_arch = "arm")]
extern crate cortex_m_semihosting;

/// ARM-specific entry point
#[cfg(target_arch = "arm")]
#[cortex_m_rt::entry]
fn main() -> ! {
    // Test semihosting to confirm execution
    use cortex_m_semihosting::hprintln;
    let _ = hprintln!("ARM main function reached!");
    
    // Direct UART test - bypass all complex initialization
    test_uart_direct();
    let _ = hprintln!("UART test completed!");
    
    // Exit cleanly using semihosting to test if semihosting works
    use cortex_m_semihosting::debug;
    debug::exit(debug::EXIT_SUCCESS);
    
    loop {
        cortex_m::asm::wfi();
    }
}

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
/// Direct UART test for LM3S6965EVB
#[cfg(target_arch = "arm")]
fn test_uart_direct() {
    // LM3S6965EVB UART0 at 0x4000C000
    const UART0_BASE: usize = 0x4000C000;
    const UARTDR: usize = UART0_BASE + 0x000; // Data register
    const UARTFR: usize = UART0_BASE + 0x018; // Flag register
    const UARTIBRD: usize = UART0_BASE + 0x024; // Integer baud rate divisor
    const UARTFBRD: usize = UART0_BASE + 0x028; // Fractional baud rate divisor
    const UARTLCRH: usize = UART0_BASE + 0x02C; // Line control register
    const UARTCTL: usize = UART0_BASE + 0x030; // Control register
    
    // Enable UART0 clock via RCGC1
    const RCGC1: usize = 0x400FE104;
    unsafe {
        let rcgc1 = core::ptr::read_volatile(RCGC1 as *const u32);
        core::ptr::write_volatile(RCGC1 as *mut u32, rcgc1 | (1 << 0));
        
        // Small delay for clock to stabilize
        for _ in 0..10000 {
            cortex_m::asm::nop();
        }
        
        // Configure UART for 115200 baud rate
        core::ptr::write_volatile(UARTIBRD as *mut u32, 8);
        core::ptr::write_volatile(UARTFBRD as *mut u32, 44);
        
        // Configure line control: 8 bits, no parity, 1 stop bit, enable FIFOs
        core::ptr::write_volatile(UARTLCRH as *mut u32, 0x70);
        
        // Enable UART, TX, RX
        core::ptr::write_volatile(UARTCTL as *mut u32, 0x301);
        
        // Small delay
        for _ in 0..10000 {
            cortex_m::asm::nop();
        }
        
        // Send test message multiple times to make it visible
        let message = b"UART TEST: Hello from ARM on LM3S6965EVB!\n";
        for _ in 0..3 {
            for &byte in message {
                // Wait for TX FIFO to have space
                while core::ptr::read_volatile(UARTFR as *const u32) & (1 << 5) != 0 {
                    cortex_m::asm::nop();
                }
                core::ptr::write_volatile(UARTDR as *mut u32, byte as u32);
            }
            // Extra delay between messages
            for _ in 0..100000 {
                cortex_m::asm::nop();
            }
        }
    }
}

/// RISC-V specific entry point  
#[cfg(target_arch = "riscv32")]
#[riscv_rt::entry]
fn main() -> ! {
    arch::early_println("RISC-V entry point reached");
    kernel_main()
}
