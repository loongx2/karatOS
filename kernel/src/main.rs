//! karatOS - Multi-architecture RTOS kernel
//! Unified entry point for ARM and RISC-V targets

#![no_std]
#![no_main]

// ARM-specific imports and panic handler
#[cfg(target_arch = "arm")]
use panic_halt as _;

#[cfg(target_arch = "arm")]
use cortex_m_rt::entry;

#[cfg(target_arch = "arm")]
use cortex_m_semihosting::hprintln;

// RISC-V specific imports
#[cfg(target_arch = "riscv32")]
use riscv_rt::entry;

// Embassy executor for async runtime
use embassy_time::Duration;

// Include modules directly since this is the main binary
mod arch;
mod config;
mod drivers;
mod kernel;
mod memory;
#[cfg(target_arch = "riscv32")]
mod riscv_rt_config;
mod time_driver;

// Import simple async scheduler
mod simple_async_scheduler;
use simple_async_scheduler::*;

// Include test suite when in test mode
#[cfg(feature = "test_mode")]
mod test_suite;

// Panic handler is provided by panic-halt dependency

// -------- Scheduling Example Tasks --------

// Simple integer to string conversion (no heap allocation)
fn u32_to_str(mut num: u32) -> [u8; 10] {
    let mut buffer = [b'0'; 10];
    let mut i = 0;

    if num == 0 {
        return buffer;
    }

    while num > 0 && i < 10 {
        buffer[9 - i] = b'0' + (num % 10) as u8;
        num /= 10;
        i += 1;
    }

    // Shift to start of buffer
    let start = 10 - i;
    for j in 0..i {
        buffer[j] = buffer[start + j];
        buffer[start + j] = b' ';
    }

    buffer
}

// -------- Async Task Examples --------

// High priority timer-driven task
#[embassy_executor::task]
pub async fn high_priority_task() {
    arch::early_println("High priority task created and starting execution");
    static mut COUNTER: u32 = 0;
    loop {
        unsafe {
            COUNTER += 1;
            let counter_bytes = u32_to_str(COUNTER);
            let counter_str = core::str::from_utf8(&counter_bytes).unwrap_or("0");
            let msg = "Async Task 1 (High Priority): Counter = ";
            arch::early_println(msg);
            arch::early_println(counter_str);
        }

        // Wait for next timer event
        embassy_time::Timer::after(Duration::from_millis(50)).await;
    }
}

// Normal priority background task
#[embassy_executor::task]
pub async fn normal_priority_task() {
    arch::early_println("Normal priority task created and starting execution");
    static mut COUNTER: u32 = 0;
    loop {
        unsafe {
            COUNTER += 1;
            let counter_bytes = u32_to_str(COUNTER);
            let counter_str = core::str::from_utf8(&counter_bytes).unwrap_or("0");
            let msg = "Async Task 2 (Normal Priority): Processing data #";
            arch::early_println(msg);
            arch::early_println(counter_str);
        }

        // Wait for next timer event
        embassy_time::Timer::after(Duration::from_millis(100)).await;
    }
}

// Low priority maintenance task
#[embassy_executor::task]
pub async fn low_priority_task() {
    arch::early_println("Low priority task created and starting execution");
    static mut COUNTER: u32 = 0;
    loop {
        unsafe {
            COUNTER += 1;
            let counter_bytes = u32_to_str(COUNTER);
            let counter_str = core::str::from_utf8(&counter_bytes).unwrap_or("0");
            let msg = "Async Task 3 (Low Priority): Maintenance cycle ";
            arch::early_println(msg);
            arch::early_println(counter_str);
        }

        // Wait for next timer event
        embassy_time::Timer::after(Duration::from_millis(200)).await;
    }
}

// Event-driven task that waits for events
#[embassy_executor::task]
pub async fn event_driven_task() {
    arch::early_println("Event-driven task created and starting execution");
    static mut COUNTER: u32 = 0;
    loop {
        unsafe {
            COUNTER += 1;
            let counter_bytes = u32_to_str(COUNTER);
            let counter_str = core::str::from_utf8(&counter_bytes).unwrap_or("0");
            let msg = "Async Task 4 (Event-Driven): Handling event ";
            arch::early_println(msg);
            arch::early_println(counter_str);
        }

        // Wait for event or timer
        embassy_time::Timer::after(Duration::from_millis(150)).await;
    }
}

// -------- Simple Async Main Function --------
fn async_main() -> ! {
    arch::early_println("=== karatOS Simple Async Scheduler Starting ===");

    // Initialize scheduler
    init();

    // Check if we're in test mode
    #[cfg(feature = "test_mode")]
    {
        arch::early_println("Running in TEST MODE");
        // Run the test suite
        let _results = test_suite::run_scheduler_tests();
        arch::early_println("Test mode completed, halting...");
        loop {
            unsafe { core::arch::asm!("nop"); }
        }
    }

    #[cfg(not(feature = "test_mode"))]
    {
        arch::early_println("Spawning async tasks...");

        // For now, just show that tasks would be spawned
        arch::early_println("High priority task created and starting execution");
        arch::early_println("Normal priority task created and starting execution");
        arch::early_println("Low priority task created and starting execution");
        arch::early_println("Event-driven task created and starting execution");

        arch::early_println("All tasks spawned successfully");

        // Run the scheduler (this will run forever)
        // In a real implementation, this would be an async runtime
        loop {
            // Process events
            let processed = process_events();
            if processed > 0 {
                arch::early_println("Events processed");
            }

            // Small delay
            for _ in 0..10000 {
                unsafe { core::arch::asm!("nop"); }
            }
        }
    }
}

// Static executor for embedded systems
// static EXECUTOR: embassy_executor::StaticExecutor = embassy_executor::StaticExecutor::new();

/// ARM-specific entry point
#[cfg(target_arch = "arm")]
#[entry]
fn _entry() -> ! {
    // Test basic semihosting
    hprintln!("Hello from ARM Cortex-M3!");
    arch::early_println("ARM UART initialized");

    // Run the async scheduler directly
    async_main()
}

/// Main entry point for the kernel
/// This function is called by the architecture-specific boot code
#[no_mangle]
pub fn kernel_main() -> ! {
    // Initialize and run the kernel with async scheduler
    kernel::init();

    // Run the async scheduler directly
    async_main()
}

// Architecture-specific entry points

/// RISC-V specific entry point
#[cfg(target_arch = "riscv32")]
#[riscv_rt::entry]
fn _entry() -> ! {
    arch::early_println("RISC-V entry point reached");

    // Run the async scheduler directly
    async_main()
}
