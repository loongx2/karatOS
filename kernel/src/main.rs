//! ============================================================================
//! MODULE : kernel binary — unified entry point for ARM and RISC-V targets
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   Boot entry, then a live demonstration of the multi-priority scheduler
//!   driven by the REAL 1 kHz tick (SysTick on ARM, CLINT on RISC-V).
//!
//! BOOT FLOW (identical on both architectures)
//!   [arch reset] -> main() -> boot_kernel()
//!     1. kernel::init()        board -> arch -> driver registry -> 1 kHz tick
//!     2. run_enhanced_scheduler_test()
//!        per cycle: arch::advance_time() drains the tick latch into MILLIS,
//!        then update_global_timer() feeds the scheduler real milliseconds —
//!        replacing the old fake `timer_counter`.
//!
//! SCHEDULER (see scheduler.rs)
//!   Cooperative, 4 priority levels (Critical > High > Normal > Low),
//!   lock-free event queues, `MAX_TASKS` slots, static allocation only.
//!   No context switching: the tick drives time/sleeps, NOT preemption —
//!   a documented Phase-0 decision (keeps ROM/RAM and ISR latency minimal).
//!
//! MEMORY BUDGET
//!   Tasks are zero-sized fn()s executed from scheduler slots; demo output
//!   goes through arch::early_println (pre-driver console path). Release
//!   footprint target: < 64 kB ROM / < 64 kB SRAM (enforced by size.sh).
//! ============================================================================

#![no_std]
#![no_main]

// ARM-specific imports and panic handler
// Host builds (`cargo check --all-targets`) need a panic handler too:
// the binary is no_std on every target.
#[cfg(not(any(
    target_arch = "arm",
    target_arch = "riscv32",
    target_arch = "riscv64",
    test
)))]
#[panic_handler]
fn host_panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[cfg(target_arch = "arm")]
use panic_halt as _;

#[cfg(target_arch = "arm")]
use cortex_m_rt::entry;

#[cfg(target_arch = "arm")]
use cortex_m_semihosting::hprintln;

// RISC-V specific imports and panic handler
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
use panic_halt as _;

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
#[allow(unused_imports)]
use riscv_rt::entry;

// Include modules directly since this is the main binary
mod arch;
mod board;
mod config;
mod dtb;
mod drivers;
mod kapi;
mod modules;
mod kernel;
mod logger;
mod memory;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
mod riscv_rt_config;

// ---------------------------------------------------------------------------
// Phase 3: embedded flash module store. The loadable module (modules/hello)
// is built BEFORE the kernel and its raw image is embedded here at compile
// time. Phase 4 moves this to a dedicated flash store region; the loader
// path (verify CRC -> copy to RAM slot -> call in) is already the real one.
// ---------------------------------------------------------------------------
#[cfg(karatos_module_image)]
static MODULE_IMAGE: &[u8] = include_bytes!(env!("KARATOS_MODULE_BIN"));

/// Load the embedded module image. Lives in the bin because the image
/// itself is embedded in the binary (lib stays image-agnostic).
#[cfg(karatos_module_image)]
fn load_embedded_module() -> Result<usize, modules::ModuleError> {
    modules::load_image(MODULE_IMAGE)
}

// Import scheduler for task management
mod scheduler;
mod store;
use scheduler::{Task, TaskPriority, EventPriority, post_priority_event, 
                add_priority_task, schedule_with_priority, 
                update_global_timer, has_ready_work, current_priority_level};

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

// -------- Enhanced Scheduling Test Tasks --------

// Task 1: Critical priority system task
fn task_critical_system() {
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        let counter_bytes = u32_to_str(COUNTER);
        let counter_str = core::str::from_utf8(&counter_bytes).unwrap_or("0");
        arch::early_println("🚨 CRITICAL: System task #");
        arch::early_println(counter_str);
        arch::early_println(" executing");
    }
}

// Task 2: High priority real-time task
fn task_high_realtime() {
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        let counter_bytes = u32_to_str(COUNTER);
        let counter_str = core::str::from_utf8(&counter_bytes).unwrap_or("0");
        arch::early_println("⚡ HIGH: Real-time task #");
        arch::early_println(counter_str);
        arch::early_println(" processing");
    }
}

// Task 3: Normal priority application task
fn task_normal_app() {
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        let counter_bytes = u32_to_str(COUNTER);
        let counter_str = core::str::from_utf8(&counter_bytes).unwrap_or("0");
        arch::early_println("📱 NORMAL: App task #");
        arch::early_println(counter_str);
        arch::early_println(" running");
    }
}

// Task 4: Low priority background task
fn task_low_background() {
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        let counter_bytes = u32_to_str(COUNTER);
        let counter_str = core::str::from_utf8(&counter_bytes).unwrap_or("0");
        arch::early_println("🔄 LOW: Background task #");
        arch::early_println(counter_str);
        arch::early_println(" cleaning");
    }
}

// Task 5: Event-driven message processing task
fn task_message_processor() {
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        let counter_bytes = u32_to_str(COUNTER);
        let counter_str = core::str::from_utf8(&counter_bytes).unwrap_or("0");
        arch::early_println("📨 EVENT: Message #");
        arch::early_println(counter_str);
        arch::early_println(" handled");
    }
}

// Task 6: Timer-based periodic task
fn task_timer_periodic() {
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        let counter_bytes = u32_to_str(COUNTER);
        let counter_str = core::str::from_utf8(&counter_bytes).unwrap_or("0");
        arch::early_println("⏱️  TIMER: Periodic #");
        arch::early_println(counter_str);
        arch::early_println(" tick");
    }
}

// -------- Enhanced Multi-Priority Scheduler Test --------
fn run_enhanced_scheduler_test() -> ! {
    arch::early_println("=== karatOS Enhanced Multi-Priority Scheduler Test ===");
    arch::early_println("Features: Priority preemption, message-passing optimization,");
    arch::early_println("lock-free queues, timer integration, architecture-agnostic");
    arch::early_println("");

    // Create tasks with different priorities
    let critical_task = Task::with_priority(1, TaskPriority::Critical);
    let high_task = Task::with_priority(2, TaskPriority::High);
    let normal_task1 = Task::with_priority(3, TaskPriority::Normal);
    let normal_task2 = Task::with_priority(4, TaskPriority::Normal);
    let low_task1 = Task::with_priority(5, TaskPriority::Low);
    let low_task2 = Task::with_priority(6, TaskPriority::Low);

    // Spawn tasks using multi-priority scheduler
    match add_priority_task(critical_task) {
        Ok(id) => {
            arch::early_println("✅ Spawned Critical System Task ID: ");
            let id_str = u32_to_str(id as u32);
            arch::early_println(core::str::from_utf8(&id_str).unwrap_or("0"));
        },
        Err(_) => arch::early_println("❌ Failed to spawn Critical Task"),
    }

    match add_priority_task(high_task) {
        Ok(id) => {
            arch::early_println("✅ Spawned High Priority Real-time Task ID: ");
            let id_str = u32_to_str(id as u32);
            arch::early_println(core::str::from_utf8(&id_str).unwrap_or("0"));
        },
        Err(_) => arch::early_println("❌ Failed to spawn High Priority Task"),
    }

    match add_priority_task(normal_task1) {
        Ok(id) => {
            arch::early_println("✅ Spawned Normal App Task ID: ");
            let id_str = u32_to_str(id as u32);
            arch::early_println(core::str::from_utf8(&id_str).unwrap_or("0"));
        },
        Err(_) => arch::early_println("❌ Failed to spawn Normal Task 1"),
    }

    match add_priority_task(normal_task2) {
        Ok(id) => {
            arch::early_println("✅ Spawned Message Processor Task ID: ");
            let id_str = u32_to_str(id as u32);
            arch::early_println(core::str::from_utf8(&id_str).unwrap_or("0"));
        },
        Err(_) => arch::early_println("❌ Failed to spawn Normal Task 2"),
    }

    match add_priority_task(low_task1) {
        Ok(id) => {
            arch::early_println("✅ Spawned Low Background Task ID: ");
            let id_str = u32_to_str(id as u32);
            arch::early_println(core::str::from_utf8(&id_str).unwrap_or("0"));
        },
        Err(_) => arch::early_println("❌ Failed to spawn Low Task 1"),
    }

    match add_priority_task(low_task2) {
        Ok(id) => {
            arch::early_println("✅ Spawned Timer Periodic Task ID: ");
            let id_str = u32_to_str(id as u32);
            arch::early_println(core::str::from_utf8(&id_str).unwrap_or("0"));
        },
        Err(_) => arch::early_println("❌ Failed to spawn Low Task 2"),
    }

    arch::early_println("");
    arch::early_println("=== Starting Multi-Priority Preemptive Scheduler ===");
    arch::early_println("Priority order: Critical > High > Normal > Low");
    arch::early_println("Features: Message-passing hot-slot, lock-free events, timers");
    arch::early_println("");

    let mut cycle_counter = 0u32;
    loop {
        cycle_counter += 1;

        // Advance the kernel clock: drain the tick latch (fed by the
        // SysTick/CLINT ISR at 1 kHz) into monotonic millis, then hand the
        // scheduler the real time-of-boot. This replaces the old fake
        // `timer_counter` — sleeps and timeouts are now honest.
        let now = arch::advance_time();
        update_global_timer(now);

        // ---- Phase 3 demo: load -> list -> retract -> list -> reload -----
        // Proves the module is EXTENSIBLE (adds code at runtime) and
        // RETRACTABLE (SRAM reclaimed, slot wiped, re-loadable afterwards).
        // Phase 4: the store (flash region provisioned by the bootloader)
        // is authoritative; the embedded image is the offline fallback.
        match cycle_counter {
            50 => {
                arch::early_println("\r\n[store] probing flash store...");
                match store::probe() {
                    Ok(st) => {
                        arch::early_println("[store] ok: board ");
                        arch::early_println(core::str::from_utf8(&u32_to_str(st.board_id() as u32)).unwrap_or("?"));
                        arch::early_println(" | entries: ");
                        arch::early_println(core::str::from_utf8(&u32_to_str(st.entry_count() as u32)).unwrap_or("?"));
                    }
                    Err(e) => {
                        let code = match e {
                            store::StoreError::NoStore => 0u32,
                            store::StoreError::BadMagic => 1,
                            store::StoreError::BadVersion => 2,
                            store::StoreError::BoardMismatch => 3,
                            store::StoreError::BadSize => 4,
                            store::StoreError::CrcMismatch => 5,
                            store::StoreError::NotFound => 6,
                            store::StoreError::LoadFailed => 7,
                        };
                        arch::early_println("[store] probe error ");
                        arch::early_println(core::str::from_utf8(&u32_to_str(code)).unwrap_or("?"));
                    }
                }
            }
            60 => {
                let from_store = matches!(store::probe(), Ok(ref st) if st.image("hello").is_ok());
                if from_store {
                    arch::early_println("\r\n[store] loading module 'hello' by name...");
                    match store::load("hello") {
                        Ok(idx) => {
                            arch::early_println("[store] loaded at registry slot ");
                            arch::early_println(core::str::from_utf8(&u32_to_str(idx as u32)).unwrap_or("?"));
                        }
                        Err(_) => arch::early_println("[store] load FAILED"),
                    }
                }
                #[cfg(karatos_module_image)]
                if !from_store {
                    arch::early_println("\r\n[mod] falling back to embedded module image...");
                    match load_embedded_module() {
                        Ok(_) => arch::early_println("[mod] embedded module loaded"),
                        Err(_) => arch::early_println("[mod] load FAILED"),
                    }
                }
            }
            100 => {
                arch::early_println("\r\n[mod] module registry:");
                modules::list();
            }
            150 => {
                arch::early_println("\r\n[mod] retracting slot 0...");
                let _ = modules::retract(0);
            }
            160 => {
                arch::early_println("\r\n[store] listing store entries:");
                match store::probe() {
                    Ok(st) => {
                        arch::early_println("[store] entries: ");
                        arch::early_println(core::str::from_utf8(&u32_to_str(st.entry_count() as u32)).unwrap_or("?"));
                    }
                    Err(_) => arch::early_println("[store] not provisioned"),
                }
            }
            200 => {
                arch::early_println("\r\n[mod] module registry after retract:");
                modules::list();
            }
            250 => {
                arch::early_println("\r\n[mod] re-loading module (extensibility is repeatable)");
                let via_store = matches!(store::probe(), Ok(ref st) if st.image("hello").is_ok());
                if via_store {
                    let _ = store::load("hello");
                }
                #[cfg(karatos_module_image)]
                if !via_store {
                    let _ = load_embedded_module();
                }
            }
            _ => {}
        }

        // Run the enhanced multi-priority scheduler
        if let Some(current_task) = schedule_with_priority() {
            let priority_level = current_priority_level();
            
            // Execute task based on ID and priority
            match (current_task.id, current_task.priority) {
                (1, TaskPriority::Critical) => {
                    task_critical_system();
                    arch::early_println(" [Critical task completed]");
                },
                (2, TaskPriority::High) => {
                    task_high_realtime();
                    arch::early_println(" [High priority task completed]");
                },
                (3, TaskPriority::Normal) => {
                    task_normal_app();
                    arch::early_println(" [Normal app task completed]");
                },
                (4, TaskPriority::Normal) => {
                    task_message_processor();
                    arch::early_println(" [Message processor completed]");
                },
                (5, TaskPriority::Low) => {
                    task_low_background();
                    arch::early_println(" [Background task completed]");
                },
                (6, TaskPriority::Low) => {
                    task_timer_periodic();
                    arch::early_println(" [Timer task completed]");
                },
                _ => {
                    arch::early_println("⚠️  Unknown task: ");
                    let id_str = u32_to_str(current_task.id as u32);
                    arch::early_println(core::str::from_utf8(&id_str).unwrap_or("?"));
                },
            }

            // Show current priority level
            let priority_str = match priority_level {
                TaskPriority::Critical => " 🚨 CRITICAL",
                TaskPriority::High => " ⚡ HIGH",
                TaskPriority::Normal => " 📱 NORMAL", 
                TaskPriority::Low => " 🔄 LOW",
            };
            arch::early_println(priority_str);
        } else {
            arch::early_println("💤 No ready tasks - CPU can sleep");
        }

        // Demonstrate event posting and priority handling
        match cycle_counter % 50 {
            5 => {
                // Post critical event (simulates interrupt)
                if post_priority_event(0x10, EventPriority::Critical) {
                    arch::early_println("🚨 Posted CRITICAL interrupt event");
                }
            },
            15 => {
                // Post high priority event (simulates real-time deadline)
                if post_priority_event(0x20, EventPriority::High) {
                    arch::early_println("⚡ Posted HIGH priority real-time event");
                }
            },
            25 => {
                // Post normal event (simulates user interaction)
                if post_priority_event(0x30, EventPriority::Normal) {
                    arch::early_println("📱 Posted NORMAL user event");
                }
            },
            35 => {
                // Post low priority event (simulates background work)
                if post_priority_event(0x40, EventPriority::Low) {
                    arch::early_println("🔄 Posted LOW background event");
                }
            },
            _ => {}
        }

        // Display scheduler statistics
        if cycle_counter % 100 == 0 {
            let (active_tasks, events, timer) = scheduler::scheduler_stats();
            
            arch::early_println("");
            arch::early_println("📊 === Scheduler Statistics ===");
            arch::early_println("Cycle: ");
            let cycle_str = u32_to_str(cycle_counter);
            arch::early_println(core::str::from_utf8(&cycle_str).unwrap_or("0"));
            
            arch::early_println(" | Active Tasks: ");
            let tasks_str = u32_to_str(active_tasks);
            arch::early_println(core::str::from_utf8(&tasks_str).unwrap_or("0"));
            
            arch::early_println(" | Events: ");
            let events_str = u32_to_str(events);
            arch::early_println(core::str::from_utf8(&events_str).unwrap_or("0"));
            
            arch::early_println(" | Timer: ");
            let timer_str = u32_to_str(timer as u32);
            arch::early_println(core::str::from_utf8(&timer_str).unwrap_or("0"));
            
            arch::early_println("");
            
            if has_ready_work() {
                arch::early_println("🟢 Scheduler has ready work");
            } else {
                arch::early_println("🔴 No ready work - entering low power mode");
            }
            arch::early_println("");
        }

        // Demonstrate preemption scenario
        if cycle_counter % 200 == 0 {
            arch::early_println("🔄 === Preemption Test Scenario ===");
            arch::early_println("Posting multiple events to test priority handling...");
            
            // Post events in reverse priority order to test preemption
            let _ = post_priority_event(0x50, EventPriority::Low);
            let _ = post_priority_event(0x51, EventPriority::Normal);
            let _ = post_priority_event(0x52, EventPriority::High);
            let _ = post_priority_event(0x53, EventPriority::Critical);
            
            arch::early_println("Posted: Low->Normal->High->Critical");
            arch::early_println("Expected execution order: Critical->High->Normal->Low");
            arch::early_println("");
        }

        // Small delay for readability (architecture-agnostic)
        for _ in 0..8000 {
            scheduler::yield_now();
        }

        // Demonstrate sleep functionality periodically
        if cycle_counter % 300 == 0 {
            arch::early_println("😴 Testing sleep functionality...");
            // Note: In a real implementation, tasks would call sleep_current()
            // Here we just demonstrate the timer update mechanism
        }
    }
}

/// Unified boot path: both architectures land here after their runtime
/// reset code. kernel::init() performs DTB probe -> board -> arch -> driver
/// registry -> tick bring-up, then the scheduler + module demo runs forever.
///
/// `dtb_addr` — Phase 2: the RISC-V boot protocol delivers the device tree
/// pointer in a1 (captured by the entry fn); ARM passes None and uses the
/// built-in board table.
fn boot_kernel(dtb_addr: Option<usize>) -> ! {
    kernel::init(dtb_addr);
    run_enhanced_scheduler_test()
}

/// ARM-specific entry point
#[cfg(target_arch = "arm")]
#[entry]
fn main() -> ! {
    // Test basic semihosting (available even before driver init)
    hprintln!("Hello from ARM Cortex-M3!");

    // LM3S QEMU provides no firmware DTB — boot with the built-in table.
    boot_kernel(None)
}

/// Main entry point for the kernel
/// This function is called by the architecture-specific boot code
#[no_mangle]
pub fn kernel_main() -> ! {
    boot_kernel(None)
}

// Architecture-specific entry points

/// RISC-V specific entry point.
/// Boot protocol registers (riscv-rt forwards them): a0 = hart id,
/// a1 = device tree blob pointer (Phase 2 runtime enumeration input).
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
#[riscv_rt::entry]
fn main(a0: usize, a1: usize, _a2: usize) -> ! {
    let _ = a0;
    arch::early_println("RISC-V entry point reached");
    boot_kernel(if a1 != 0 { Some(a1) } else { None })
}
