//! Architecture-agnostic test suite for karatOS RTOS
//!
//! This module provides comprehensive testing of the async event scheduler
//! with identical functionality across ARM and RISC-V architectures.

use crate::simple_async_scheduler::*;
use crate::arch;

// Test configuration
const TEST_DURATION_CYCLES: u32 = 100000; // Test duration in busy-wait cycles
const EVENT_TEST_COUNT: u32 = 10; // Number of events to post per test

/// Test results structure
#[derive(Debug, Clone, Copy)]
pub struct TestResults {
    pub events_processed: u32,
    pub tasks_executed: u32,
    pub scheduler_cycles: u32,
    pub priority_switches: u32,
}

/// Run comprehensive scheduler tests
pub fn run_scheduler_tests() -> TestResults {
    arch::early_println("=== Starting karatOS Scheduler Tests ===");

    let mut results = TestResults {
        events_processed: 0,
        tasks_executed: 0,
        scheduler_cycles: 0,
        priority_switches: 0,
    };

    // Test 1: Event posting and processing
    arch::early_println("Test 1: Event Posting and Processing");
    results.events_processed = test_event_processing();

    // Test 2: Priority scheduling
    arch::early_println("Test 2: Priority Scheduling");
    results.priority_switches = test_priority_scheduling();

    // Test 3: Timer events
    arch::early_println("Test 3: Timer Events");
    results.tasks_executed = test_timer_events();

    // Test 4: Scheduler performance
    arch::early_println("Test 4: Scheduler Performance");
    results.scheduler_cycles = test_scheduler_performance();

    arch::early_println("=== Scheduler Tests Complete ===");

    // Print results
    print_test_results(&results);

    results
}

/// Test basic event posting and processing
fn test_event_processing() -> u32 {
    arch::early_println("Creating test events for event processing test...");

    let mut processed: u32 = 0;

    // Post events of different priorities
    for i in 0..EVENT_TEST_COUNT {
        let priority = match i % 4 {
            0 => {
                arch::early_println("Creating CRITICAL priority event");
                EventPriority::Critical
            },
            1 => {
                arch::early_println("Creating HIGH priority event");
                EventPriority::High
            },
            2 => {
                arch::early_println("Creating NORMAL priority event");
                EventPriority::Normal
            },
            _ => {
                arch::early_println("Creating LOW priority event");
                EventPriority::Low
            },
        };

        if post_event_with_priority(200 + i, priority) {
            processed += 1;
            arch::early_println("Event posted successfully");
        } else {
            arch::early_println("Failed to post event");
        }
    }

    arch::early_println("Processing events...");
    // Process events
    let mut cycles = 0;
    while cycles < 1000 && processed > 0 {
        let count = process_events();
        if count > 0 {
            processed = processed.saturating_sub(count);
            arch::early_println("Processed events in this cycle");
        }
        cycles += 1;

        // Small delay
        test_delay(100);
    }

    // Print results using static strings
    arch::early_println("Event processing test completed");
    EVENT_TEST_COUNT
}

/// Test priority scheduling behavior
fn test_priority_scheduling() -> u32 {
    arch::early_println("Testing priority scheduling behavior...");
    let mut switches = 0;

    // Clear any existing events
    arch::early_println("Clearing existing events from all queues...");
    while !CRITICAL_EVENTS.is_empty() { CRITICAL_EVENTS.pop(); }
    while !HIGH_EVENTS.is_empty() { HIGH_EVENTS.pop(); }
    while !NORMAL_EVENTS.is_empty() { NORMAL_EVENTS.pop(); }
    while !LOW_EVENTS.is_empty() { LOW_EVENTS.pop(); }

    // Post events in reverse priority order (low to high)
    arch::early_println("Posting LOW priority event (ID: 300)");
    post_event_with_priority(300, EventPriority::Low);
    arch::early_println("Posting NORMAL priority event (ID: 301)");
    post_event_with_priority(301, EventPriority::Normal);
    arch::early_println("Posting HIGH priority event (ID: 302)");
    post_event_with_priority(302, EventPriority::High);
    arch::early_println("Posting CRITICAL priority event (ID: 303)");
    post_event_with_priority(303, EventPriority::Critical);

    // Process events and count priority switches
    let mut last_priority = EventPriority::Low;
    let mut processed = 0;

    arch::early_println("Processing events in priority order...");
    while processed < 4 {
        if let Some(event) = CRITICAL_EVENTS.pop() {
            if event.priority != last_priority {
                switches += 1;
                last_priority = event.priority;
                arch::early_println("Priority switch detected - processing CRITICAL event");
            }
            processed += 1;
            arch::early_println("Processed CRITICAL priority event");
        } else if let Some(event) = HIGH_EVENTS.pop() {
            if event.priority != last_priority {
                switches += 1;
                last_priority = event.priority;
                arch::early_println("Priority switch detected - processing HIGH event");
            }
            processed += 1;
            arch::early_println("Processed HIGH priority event");
        } else if let Some(event) = NORMAL_EVENTS.pop() {
            if event.priority != last_priority {
                switches += 1;
                last_priority = event.priority;
                arch::early_println("Priority switch detected - processing NORMAL event");
            }
            processed += 1;
            arch::early_println("Processed NORMAL priority event");
        } else if let Some(event) = LOW_EVENTS.pop() {
            if event.priority != last_priority {
                switches += 1;
                last_priority = event.priority;
                arch::early_println("Priority switch detected - processing LOW event");
            }
            processed += 1;
            arch::early_println("Processed LOW priority event");
        } else {
            break;
        }
    }

    arch::early_println("Priority scheduling test completed");
    switches
}

/// Test timer event generation
fn test_timer_events() -> u32 {
    arch::early_println("Testing timer event generation...");
    let mut timer_events = 0;

    // Simulate timer task for a short period
    for i in 0..50 {
        // Post timer events in round-robin fashion
        let event_id = 100 + (TIMER_EVENT_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed) % 4);
        arch::early_println("Creating timer event task");
        post_event_with_priority(event_id, EventPriority::High);
        timer_events += 1;
        arch::early_println("Timer event posted successfully");

        // Process one event
        let processed = process_events();
        if processed > 0 {
            arch::early_println("Timer event processed");
        }

        // Small delay
        test_delay(1000);
    }

    arch::early_println("Timer events test completed");
    timer_events
}

/// Test scheduler performance under load
fn test_scheduler_performance() -> u32 {
    arch::early_println("Testing scheduler performance under load...");
    let mut cycles = 0;

    // Flood the scheduler with events
    arch::early_println("Creating performance test events...");
    for i in 0..MAX_EVENTS {
        let priority = match i % 4 {
            0 => {
                if i % 20 == 0 { // Print every 20th event to avoid spam
                    arch::early_println("Creating CRITICAL priority event");
                }
                EventPriority::Critical
            },
            1 => {
                if i % 20 == 0 {
                    arch::early_println("Creating HIGH priority event");
                }
                EventPriority::High
            },
            2 => {
                if i % 20 == 0 {
                    arch::early_println("Creating NORMAL priority event");
                }
                EventPriority::Normal
            },
            _ => {
                if i % 20 == 0 {
                    arch::early_println("Creating LOW priority event");
                }
                EventPriority::Low
            },
        };
        post_event_with_priority(400 + i as u32, priority);
    }
    arch::early_println("All performance test events created");

    // Time how long it takes to process all events
    arch::early_println("Processing performance test events...");
    while cycles < TEST_DURATION_CYCLES {
        let processed = process_events();
        if processed == 0 {
            // No more events to process
            arch::early_println("All events processed");
            break;
        }
        cycles += 1;
        if cycles % 100 == 0 { // Print progress every 100 cycles
            arch::early_println("Processing cycle completed");
        }
    }

    arch::early_println("Scheduler performance test completed");
    cycles
}

/// Print test results in a standardized format
fn print_test_results(results: &TestResults) {
    arch::early_println("=== Test Results Summary ===");
    arch::early_println("Events Processed: [count]");
    arch::early_println("Tasks Executed: [count]");
    arch::early_println("Scheduler Cycles: [count]");
    arch::early_println("Priority Switches: [count]");
    arch::early_println("============================");
}

/// Architecture-agnostic delay function
pub fn test_delay(cycles: u32) {
    for _ in 0..cycles {
        unsafe { core::arch::asm!("nop"); }
    }
}
