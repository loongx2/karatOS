//! Platform-Agnostic Kernel Main
//! Simplified kernel that demonstrates the refactored architecture

#![no_std]
#![no_main]

mod arch;
mod drivers;
mod config;

use config::{get_device_config, init_platform};
use drivers::DriverManager;

// Platform-specific entry point handling
#[cfg(target_arch = "arm")]
use cortex_m_rt::entry;

// Platform-specific panic handlers
use panic_halt as _;

// Global kernel state
static mut DRIVER_MANAGER: Option<DriverManager> = None;

/// Initialize the kernel with platform-specific configuration
fn kernel_init() -> Result<(), &'static str> {
    // Initialize platform-specific features
    init_platform();
    arch::arch_println("Platform initialized");
    
    // Get device configuration for this platform
    let device_config = get_device_config();
    arch::arch_println("Device configuration loaded");
    
    // Initialize driver manager
    let mut driver_manager = DriverManager::new();
    driver_manager.init_drivers(&device_config)?;
    
    // Store driver manager globally
    unsafe {
        DRIVER_MANAGER = Some(driver_manager);
    }
    
    arch::arch_println("Kernel initialized successfully");
    Ok(())
}

/// Get the global driver manager
fn get_driver_manager() -> Option<&'static mut DriverManager> {
    unsafe { DRIVER_MANAGER.as_mut() }
}

/// Simple kernel main loop
fn kernel_main_loop() {
    arch::arch_println("=== Multi-Architecture Rust RTOS ===");
    
    // Print architecture info
    #[cfg(target_arch = "arm")]
    arch::arch_println("Architecture: ARM");
    #[cfg(target_arch = "riscv32")]
    arch::arch_println("Architecture: RISC-V");
    
    // Print device configuration
    let _config = get_device_config();
    arch::arch_println("Device Configuration:");
    
    // Simple demonstration using the UART driver
    if let Some(dm) = get_driver_manager() {
        if let Some(uart) = dm.uart() {
            uart.write_str("UART driver working!\n");
            uart.write_str("Multi-architecture kernel successfully running.\n");
        }
    }
    
    let mut counter = 0u32;
    loop {
        // Simple heartbeat
        if counter % 1000000 == 0 {
            if let Some(dm) = get_driver_manager() {
                if let Some(uart) = dm.uart() {
                    uart.write_str("Heartbeat ");
                    
                    // Simple decimal output
                    let mut n = counter / 1000000;
                    if n == 0 {
                        uart.write_char(b'0');
                    } else {
                        let mut digits = heapless::Vec::<u8, 10>::new();
                        while n > 0 {
                            let _ = digits.push((n % 10) as u8 + b'0');
                            n /= 10;
                        }
                        // Print digits in reverse order
                        for &digit in digits.iter().rev() {
                            uart.write_char(digit);
                        }
                    }
                    uart.write_char(b'\n');
                }
            }
        }
        
        counter = counter.wrapping_add(1);
        
        // Yield to prevent 100% CPU usage in emulation
        arch::arch_yield();
    }
}

/// Main kernel entry point - works for all architectures
// ARM entry point using cortex-m-rt
#[cfg(target_arch = "arm")]
#[entry]
fn main() -> ! {
    kernel_entry()
}

// RISC-V entry point - custom implementation
#[cfg(target_arch = "riscv32")]
#[no_mangle]
#[link_section = ".text._start"]
pub extern "C" fn _start() -> ! {
    kernel_entry()
}

fn kernel_entry() -> ! {
    // Initialize kernel
    match kernel_init() {
        Ok(()) => {
            // Run main kernel loop
            kernel_main_loop();
        }
        Err(err) => {
            arch::arch_println("Kernel initialization failed:");
            arch::arch_println(err);
        }
    }
    
    // Shutdown
    arch::arch_println("Kernel shutdown complete");
    arch::arch_shutdown()
}
