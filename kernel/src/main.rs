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

// Include modules directly since this is the main binary
mod arch;
mod config;
mod drivers;
mod kernel;
mod memory;
#[cfg(target_arch = "riscv32")]
mod riscv_rt_config;

// Import scheduler for task management
mod scheduler;
use scheduler::{Task, EventPriority, post_event_with_priority, add_task};

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

// -------- Scheduling Example Tasks --------

// Task 1: High priority periodic task
fn task_high_priority() {
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        let counter_bytes = u32_to_str(COUNTER);
        let counter_str = core::str::from_utf8(&counter_bytes).unwrap_or("0");
        let msg = "Task 1 (High Priority): Counter = ";
        arch::early_println(msg);
        arch::early_println(counter_str);
    }
}

// Task 2: Normal priority background task
fn task_normal_priority() {
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        let counter_bytes = u32_to_str(COUNTER);
        let counter_str = core::str::from_utf8(&counter_bytes).unwrap_or("0");
        let msg = "Task 2 (Normal Priority): Processing data #";
        arch::early_println(msg);
        arch::early_println(counter_str);
    }
}

// Task 3: Low priority maintenance task
fn task_low_priority() {
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        let counter_bytes = u32_to_str(COUNTER);
        let counter_str = core::str::from_utf8(&counter_bytes).unwrap_or("0");
        let msg = "Task 3 (Low Priority): Maintenance cycle ";
        arch::early_println(msg);
        arch::early_println(counter_str);
    }
}

// Task 4: Event-driven task that waits for events
fn task_event_driven() {
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        let counter_bytes = u32_to_str(COUNTER);
        let counter_str = core::str::from_utf8(&counter_bytes).unwrap_or("0");
        let msg = "Task 4 (Event-Driven): Handling event ";
        arch::early_println(msg);
        arch::early_println(counter_str);
    }
}

// -------- Main Scheduling Loop --------
fn run_scheduler_example() -> ! {
    arch::early_println("=== karatOS Scheduler Example Starting ===");

    // Create and spawn tasks
    let task1 = Task::new(1);
    let task2 = Task::new(2);
    let task3 = Task::new(3);
    let task4 = Task::new(4);

    // Spawn tasks with different priorities
    if let Ok(id1) = add_task(task1) {
        let id_bytes = u32_to_str(id1 as u32);
        let id_str = core::str::from_utf8(&id_bytes).unwrap_or("0");
        arch::early_println("Spawned Task 1 (High Priority) with ID: ");
        arch::early_println(id_str);
    }

    if let Ok(id2) = add_task(task2) {
        let id_bytes = u32_to_str(id2 as u32);
        let id_str = core::str::from_utf8(&id_bytes).unwrap_or("0");
        arch::early_println("Spawned Task 2 (Normal Priority) with ID: ");
        arch::early_println(id_str);
    }

    if let Ok(id3) = add_task(task3) {
        let id_bytes = u32_to_str(id3 as u32);
        let id_str = core::str::from_utf8(&id_bytes).unwrap_or("0");
        arch::early_println("Spawned Task 3 (Low Priority) with ID: ");
        arch::early_println(id_str);
    }

    if let Ok(id4) = add_task(task4) {
        let id_bytes = u32_to_str(id4 as u32);
        let id_str = core::str::from_utf8(&id_bytes).unwrap_or("0");
        arch::early_println("Spawned Task 4 (Event-Driven) with ID: ");
        arch::early_println(id_str);
    }

    arch::early_println("=== All Tasks Spawned, Starting Round-Robin Scheduler ===");

    let mut cycle_counter = 0u32;
    let mut current_task_id = 0u32;

    loop {
        cycle_counter += 1;
        current_task_id = (current_task_id % 4) + 1; // Cycle through tasks 1-4

        // Execute the current task
        match current_task_id {
            1 => {
                task_high_priority();
                arch::early_println(" [Task 1 completed]");
            },
            2 => {
                task_normal_priority();
                arch::early_println(" [Task 2 completed]");
            },
            3 => {
                task_low_priority();
                arch::early_println(" [Task 3 completed]");
            },
            4 => {
                task_event_driven();
                arch::early_println(" [Task 4 completed]");
            },
            _ => arch::early_println("Unknown task ID"),
        }

        // Every 10 cycles, post an event to demonstrate event handling
        if cycle_counter % 10 == 0 {
            let posted = post_event_with_priority(100, EventPriority::High);
            if posted {
                arch::early_println("=== Posted high-priority event ===");
            }
        }

        // Every 25 cycles, post a normal event
        if cycle_counter % 25 == 0 {
            let posted = post_event_with_priority(200, EventPriority::Normal);
            if posted {
                arch::early_println("=== Posted normal-priority event ===");
            }
        }

        // Print scheduler status every 50 cycles
        if cycle_counter % 50 == 0 {
            let cycle_bytes = u32_to_str(cycle_counter);
            let cycle_str = core::str::from_utf8(&cycle_bytes).unwrap_or("0");
            arch::early_println("=== Scheduler cycle: ");
            arch::early_println(cycle_str);
            arch::early_println(" ===");
        }

        // Small delay between task switches
        for _ in 0..5000 {
            unsafe { core::arch::asm!("nop"); }
        }
    }
}

/// ARM-specific entry point
#[cfg(target_arch = "arm")]
#[entry]
fn main() -> ! {
    // Test basic semihosting
    hprintln!("Hello from ARM Cortex-M3!");
    arch::early_println("ARM UART initialized");

    // Run the scheduler example instead of exiting
    run_scheduler_example()
}

/// Main entry point for the kernel
/// This function is called by the architecture-specific boot code
#[no_mangle]
pub fn kernel_main() -> ! {
    // Initialize and run the kernel with scheduler example
    kernel::init();
    run_scheduler_example()
}

// Architecture-specific entry points

/// RISC-V specific entry point
#[cfg(target_arch = "riscv32")]
#[riscv_rt::entry]
fn main() -> ! {
    arch::early_println("RISC-V entry point reached");
    run_scheduler_example()
}
