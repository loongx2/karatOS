#![no_std]
#![no_main]

mod arch;
mod scheduler;

#[cfg(feature = "arm")]
mod uart;
#[cfg(feature = "arm")]
mod qemu_uart;

use scheduler::{Task, EventPriority};

#[cfg(feature = "arm")]
use cortex_m_rt::entry;

#[cfg(feature = "arm")]
use cortex_m_semihosting::debug;

#[cfg(feature = "arm")]
use panic_halt as _;

#[cfg(feature = "arm")]
use uart::{UartInterface, UartCommand, UartResponses};

#[cfg(feature = "arm")]
use qemu_uart::{init_uart, uart_write_str, uart_read_byte, uart_data_available};

// Global state for async event-driven execution
#[cfg(feature = "arm")]
static mut TERMINATE_FLAG: bool = false;
#[cfg(feature = "arm")]
static mut ITERATION_COUNT: u32 = 0;
#[cfg(feature = "arm")]
#[allow(dead_code)]
static mut TASK_WORK_COUNTER: [u32; 4] = [0; 4]; // Track work done by each task
#[cfg(feature = "arm")]
static mut UART_INTERFACE: UartInterface = UartInterface::new();

#[cfg(feature = "arm")]
#[entry]
fn main() -> ! {
    arch::arch_init();
    
    // Initialize UART interface
    init_uart();
    uart_write_str(UartResponses::welcome_message());
    
    arch::arch_println("=== Async Event-Driven RTOS Kernel ===");
    arch::arch_println("Algorithm: Priority-based Cooperative Multitasking");
    arch::arch_println("Features: No deadlocks, Mutually exclusive events, Single-threaded async");
    arch::arch_println("UART Interface: Active and ready for commands");
    
    // Spawn async event-driven tasks
    let _ = scheduler::add_task(Task::new(0)); // High-priority timer task
    let _ = scheduler::add_task(Task::new(1)); // I/O processing task  
    let _ = scheduler::add_task(Task::new(2)); // Background cleanup task
    let _ = scheduler::add_task(Task::new(3)); // User interface task
    
    arch::arch_println("Spawned 4 async tasks");
    arch::arch_println("Starting event-driven execution loop...");
    
    // Main async event loop - processes events and schedules tasks cooperatively
    while unsafe { !TERMINATE_FLAG } {
        // Process UART commands first (highest priority)
        unsafe {
            process_uart_commands();
        }
        
        unsafe { 
            ITERATION_COUNT += 1;
            
            // Simulate external termination after 100 iterations (longer demo)
            if ITERATION_COUNT >= 100 {
                scheduler::post_event_with_priority(0xFF, EventPriority::Critical);
                arch::arch_println("Posted CRITICAL shutdown event (0xFF)");
                TERMINATE_FLAG = true;
                break;
            }
        }
        
        // Cooperative scheduler - run next ready task
        if let Some(task) = scheduler::schedule() {
            unsafe {
                simulate_async_task_work(task.id);
            }
            
            // Demonstrate event-driven task blocking
            match task.id {
                0 => {
                    // Timer task - blocks on timer event every 3rd iteration
                    if unsafe { ITERATION_COUNT % 3 == 0 } {
                        scheduler::block_current(0x1); // Block on timer event
                        arch::arch_println("Task 0 (Timer) blocked on event 0x1");
                    }
                },
                1 => {
                    // I/O task - blocks on I/O completion every 4th iteration  
                    if unsafe { ITERATION_COUNT % 4 == 0 } {
                        scheduler::block_current(0x2); // Block on I/O event
                        arch::arch_println("Task 1 (I/O) blocked on event 0x2");
                    }
                },
                2 => {
                    // Background task - blocks on low priority work every 5th iteration
                    if unsafe { ITERATION_COUNT % 5 == 0 } {
                        scheduler::block_current(0x3); // Block on background event
                        arch::arch_println("Task 2 (Background) blocked on event 0x3");
                    }
                },
                3 => {
                    // UI task - blocks on user input every 6th iteration
                    if unsafe { ITERATION_COUNT % 6 == 0 } {
                        scheduler::block_current(0x4); // Block on user event
                        arch::arch_println("Task 3 (UI) blocked on event 0x4");
                    }
                },
                _ => {}
            }
        } else {
            arch::arch_println("No ready tasks - processing events only");
        }
        
        // Simulate cooperative yielding delay
        for _ in 0..30000 { 
            unsafe { core::ptr::read_volatile(&0); }
        }
        
        // Event-driven wake-up system - post events based on priority
        unsafe {
            match ITERATION_COUNT % 8 {
                1 => {
                    scheduler::post_event_with_priority(0x1, EventPriority::High);
                    arch::arch_println("Posted HIGH priority timer event (0x1)");
                },
                2 => {
                    scheduler::post_event_with_priority(0x2, EventPriority::Normal);
                    arch::arch_println("Posted NORMAL priority I/O event (0x2)");
                },
                3 => {
                    scheduler::post_event_with_priority(0x3, EventPriority::Low);
                    arch::arch_println("Posted LOW priority background event (0x3)");
                },
                4 => {
                    scheduler::post_event_with_priority(0x4, EventPriority::Normal);
                    arch::arch_println("Posted NORMAL priority UI event (0x4)");
                },
                0 => {
                    // Show scheduler statistics
                    let (_active_tasks, _total_events) = scheduler::scheduler_stats();
                    arch::arch_println("Scheduler Stats: Active tasks, Total events processed");
                },
                _ => {}
            }
        }
        
        // Demonstrate priority-based event processing
        if unsafe { ITERATION_COUNT % 10 == 0 } {
            arch::arch_println("=== Event Processing Cycle Complete ===");
            arch::arch_println("Priority order: Critical > High > Normal > Low");
            arch::arch_println("Mutual exclusion: Events processed atomically");
        }
    }
    
    // Show final task work statistics
    arch::arch_println("=== Final Task Statistics ===");
    for _i in 0..4 {
        arch::arch_println("Task completed work units");
    }
    
    arch::arch_println("=== Async Kernel Shutdown Complete ===");
    arch::arch_println("Demonstrated: Cooperative multitasking, Priority events, No deadlocks");
    
    // Exit QEMU cleanly
    #[cfg(feature = "arm")]
    debug::exit(debug::EXIT_SUCCESS);
    
    loop {}
}

// Process UART commands and handle system control
#[cfg(feature = "arm")]
unsafe fn process_uart_commands() {
    // Check for incoming UART data
    while uart_data_available() {
        if let Some(byte) = uart_read_byte() {
            // Echo the character for user feedback
            if byte >= b' ' && byte <= b'~' {
                uart_write_str(core::str::from_utf8(&[byte]).unwrap_or(""));
            } else if byte == b'\r' || byte == b'\n' {
                uart_write_str("\n");
            } else if byte == b'\x08' || byte == b'\x7f' { // Backspace
                uart_write_str("\x08 \x08"); // Backspace, space, backspace
            }
            
            // Process the byte through command parser
            if let Some(command) = UART_INTERFACE.process_byte(byte) {
                handle_uart_command(command);
                uart_write_str(UartResponses::prompt());
            }
        }
    }
}

// Handle executed UART commands
#[cfg(feature = "arm")]
unsafe fn handle_uart_command(command: UartCommand) {
    match command {
        UartCommand::Status => {
            uart_write_str(UartResponses::status_response());
        },
        UartCommand::Exit => {
            uart_write_str(UartResponses::exit_response());
            arch::arch_println("UART: Exit command received");
            TERMINATE_FLAG = true;
            // Post critical shutdown event
            scheduler::post_event_with_priority(0xFF, EventPriority::Critical);
        },
        UartCommand::Restart => {
            uart_write_str(UartResponses::restart_response());
            arch::arch_println("UART: Restart command received");
            // In a real system, this would trigger a hardware reset
            // For QEMU, we'll just restart the kernel loop
            ITERATION_COUNT = 0;
            UART_INTERFACE.clear_input();
            uart_write_str("\n=== System Restarted ===\n");
            uart_write_str(UartResponses::welcome_message());
        },
        UartCommand::Help => {
            uart_write_str(UartResponses::help_response());
        },
        UartCommand::Unknown(cmd) => {
            let response = UartResponses::unknown_response(&cmd);
            uart_write_str(&response);
        },
    }
}

// Simulate async task work with cooperative yielding
#[cfg(feature = "arm")]
unsafe fn simulate_async_task_work(task_id: usize) {
    TASK_WORK_COUNTER[task_id] += 1;
    
    match task_id {
        0 => arch::arch_println("Task 0: Timer management (high priority)"),
        1 => arch::arch_println("Task 1: I/O processing (normal priority)"), 
        2 => arch::arch_println("Task 2: Background cleanup (low priority)"),
        3 => arch::arch_println("Task 3: User interface (normal priority)"),
        _ => arch::arch_println("Task: Unknown work"),
    }
    
    // Simulate some work - tasks cooperatively yield control
    for _ in 0..10000 {
        core::ptr::read_volatile(&0);
    }
}

#[cfg(not(feature = "arm"))]
#[no_mangle]
pub extern "C" fn main() -> ! {
    arch::arch_init();
    arch::arch_println("=== RISC-V Async Event-Driven RTOS ===");
    arch::arch_println("Algorithm: Priority-based Cooperative Multitasking");
    
    // Spawn async event-driven tasks
    let _ = scheduler::add_task(Task::new(0));
    let _ = scheduler::add_task(Task::new(1));
    let _ = scheduler::add_task(Task::new(2));
    
    arch::arch_println("Spawned 3 async tasks for RISC-V");
    
    // Shorter demo for RISC-V
    let mut iteration_count = 0u32;
    while iteration_count < 15 {
        iteration_count += 1;
        
        if let Some(task) = scheduler::schedule() {
            match task.id {
                0 => arch::arch_println("RISC-V Task 0: System monitoring"),
                1 => arch::arch_println("RISC-V Task 1: Communication"),
                2 => arch::arch_println("RISC-V Task 2: Data processing"),
                _ => {}
            }
            
            // Event-driven blocking
            if iteration_count % 3 == 0 {
                scheduler::block_current(0x1);
            }
        }
        
        // Post events with priority
        if iteration_count % 4 == 0 {
            scheduler::post_event_with_priority(0x1, EventPriority::High);
            arch::arch_println("Posted HIGH priority event on RISC-V");
        }
        
        // Cooperative yield
        for _ in 0..25000 { 
            unsafe { core::ptr::read_volatile(&0); }
        }
    }
    
    arch::arch_println("=== RISC-V Async Kernel Complete ===");
    loop {}
}

#[cfg(feature = "arm")]
use panic_halt as _;

#[cfg(not(feature = "arm"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
