#![no_std]
#![no_main]

mod arch;
mod scheduler;

use scheduler::Task;

#[cfg(feature = "arm")]
use cortex_m_rt::entry;

#[cfg(feature = "arm")]
use cortex_m_semihosting::debug;

#[cfg(feature = "arm")]
use panic_halt as _;

// Global flag for termination (simulating external command)
static mut TERMINATE_FLAG: bool = false;
static mut ITERATION_COUNT: u32 = 0;

#[cfg(feature = "arm")]
#[entry]
fn main() -> ! {
    arch::arch_init();
    arch::arch_println("=== ARM RTOS Kernel Starting ===");
    
    // Add initial tasks to scheduler
    let _ = scheduler::add_task(Task::new(0));
    let _ = scheduler::add_task(Task::new(1));
    
    arch::arch_println("Tasks added to scheduler");
    arch::arch_println("Entering main loop (will terminate after 10 iterations)");
    
    // Main kernel loop - runs until external termination command
    while unsafe { !TERMINATE_FLAG } {
        unsafe { 
            ITERATION_COUNT += 1;
            
            // Simulate external termination command after 10 iterations
            if ITERATION_COUNT >= 10 {
                TERMINATE_FLAG = true;
                arch::arch_println("External termination command received!");
                break;
            }
        }
        
        // Print current iteration
        arch::arch_println("Kernel loop iteration");
        
        // Run scheduler
        if let Some(_task) = scheduler::schedule() {
            arch::arch_println("Task scheduled and executed");
            // Simulate task blocking on event 0x1
            scheduler::block_current(0x1);
        } else {
            arch::arch_println("No tasks ready, idling...");
        }
        
        // Simulate some work/delay
        for _ in 0..50000 { 
            unsafe { core::ptr::read_volatile(&0); }
        }
        
        // Simulate posting events occasionally to unblock tasks
        unsafe {
            if ITERATION_COUNT % 3 == 0 {
                arch::arch_println("Posting event 0x1 to unblock tasks");
                scheduler::post_event(0x1);
            }
        }
    }
    
    arch::arch_println("=== Kernel terminating gracefully ===");
    
    // Exit QEMU cleanly
    #[cfg(feature = "arm")]
    debug::exit(debug::EXIT_SUCCESS);
    
    loop {}
}

#[cfg(not(feature = "arm"))]
#[no_mangle]
pub extern "C" fn main() -> ! {
    arch::arch_init();
    arch::arch_println("=== RISC-V RTOS Kernel Starting ===");
    
    // Add initial tasks to scheduler
    let _ = scheduler::add_task(Task::new(0));
    let _ = scheduler::add_task(Task::new(1));
    
    arch::arch_println("Tasks added to scheduler");
    arch::arch_println("Entering main loop (will terminate after 10 iterations)");
    
    // Main kernel loop - runs until external termination command
    let mut iteration_count = 0u32;
    while iteration_count < 10 {
        iteration_count += 1;
        
        // Print current iteration
        arch::arch_println("Kernel loop iteration");
        
        // Run scheduler
        if let Some(_task) = scheduler::schedule() {
            arch::arch_println("Task scheduled and executed");
            // Simulate task blocking on event 0x1
            scheduler::block_current(0x1);
        } else {
            arch::arch_println("No tasks ready, idling...");
        }
        
        // Simulate some work/delay
        for _ in 0..50000 { 
            unsafe { core::ptr::read_volatile(&0); }
        }
        
        // Simulate posting events occasionally to unblock tasks
        if iteration_count % 3 == 0 {
            arch::arch_println("Posting event 0x1 to unblock tasks");
            scheduler::post_event(0x1);
        }
    }
    
    arch::arch_println("=== Kernel terminating gracefully ===");
    loop {}
}

#[cfg(feature = "arm")]
use panic_halt as _;

#[cfg(not(feature = "arm"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
