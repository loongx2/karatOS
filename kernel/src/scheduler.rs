//! Simple event-driven, interrupt-oriented scheduler (cooperative pre-emption hook).
//! This is a minimal illustrative design for a no_std RTOS prototype.
//! Goals: fixed-size task table, ready queue, tasks block on event bitmasks.
//! Interrupt handlers post events to wake tasks.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

pub const MAX_TASKS: usize = 4;
pub const READY_QUEUE_CAP: usize = 8; // small ring buffer for runnable tasks

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TaskState {
    Ready,
    Running,
    Waiting(u32), // waiting on any matching event bit(s)
    #[allow(dead_code)]
    Finished,
}

#[derive(Copy, Clone, Debug)]
pub struct Task {
    #[allow(dead_code)]
    pub id: usize,
    pub state: TaskState,
    pub waiting_mask: u32,
    // In a real RTOS we'd store stack pointer, registers, priority, etc.
}

impl Task {
    pub const fn new(id: usize) -> Self {
        Task { id, state: TaskState::Ready, waiting_mask: 0 }
    }
}

struct ReadyQueue {
    buf: [usize; READY_QUEUE_CAP],
    head: usize,
    tail: usize,
    len: usize,
}

impl ReadyQueue {
    const fn new() -> Self { Self { buf: [0; READY_QUEUE_CAP], head: 0, tail: 0, len: 0 } }
    fn push(&mut self, id: usize) {
        if self.len == READY_QUEUE_CAP { return; } // drop if full (could track overflow)
        self.buf[self.tail] = id;
        self.tail = (self.tail + 1) % READY_QUEUE_CAP;
        self.len += 1;
    }
    fn pop(&mut self) -> Option<usize> {
        if self.len == 0 { return None; }
        let id = self.buf[self.head];
        self.head = (self.head + 1) % READY_QUEUE_CAP;
        self.len -= 1;
        Some(id)
    }
}

pub struct Scheduler {
    tasks: [Option<Task>; MAX_TASKS],
    current: Option<usize>,
    ready: ReadyQueue,
    needs_resched: AtomicBool,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            tasks: [None, None, None, None],
            current: None,
            ready: ReadyQueue::new(),
            needs_resched: AtomicBool::new(false),
        }
    }

    pub fn add_task(&mut self, task: Task) -> Result<usize, ()> {
        for (i, slot) in self.tasks.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(task);
                self.ready.push(i);
                return Ok(i);
            }
        }
        Err(())
    }

    pub fn block_current_on(&mut self, event_mask: u32) {
        if let Some(cur) = self.current {
            if let Some(ref mut t) = self.tasks[cur] {
                t.state = TaskState::Waiting(event_mask);
                t.waiting_mask = event_mask;
            }
            self.current = None;
            self.needs_resched.store(true, Ordering::Release);
        }
    }

    pub fn post_event(&mut self, events: u32) {
        // Wake tasks whose waiting_mask intersects with events.
        for (idx, opt) in self.tasks.iter_mut().enumerate() {
            if let Some(ref mut t) = opt {
                if let TaskState::Waiting(mask) = t.state {
                    if mask & events != 0 {
                        t.state = TaskState::Ready;
                        t.waiting_mask = 0;
                        self.ready.push(idx);
                    }
                }
            }
        }
        self.needs_resched.store(true, Ordering::Release);
    }

    pub fn schedule(&mut self) -> Option<&Task> {
        if self.current.is_none() || self.needs_resched.swap(false, Ordering::AcqRel) {
            if let Some(next_id) = self.ready.pop() {
                // mark previously running task as Ready if still runnable
                if let Some(prev_id) = self.current {
                    if let Some(ref mut prev_t) = self.tasks[prev_id] {
                        if matches!(prev_t.state, TaskState::Running) {
                            prev_t.state = TaskState::Ready;
                            self.ready.push(prev_id);
                        }
                    }
                }
                if let Some(ref mut t) = self.tasks[next_id] {
                    t.state = TaskState::Running;
                }
                self.current = Some(next_id);
            }
        }
        self.current.and_then(|id| self.tasks[id].as_ref())
    }

    #[allow(dead_code)]
    pub fn current_task(&self) -> Option<&Task> {
        self.current.and_then(|id| self.tasks[id].as_ref())
    }
}

// -------- Global singleton (unsafe static) --------
struct SchedulerCell(UnsafeCell<Scheduler>);
unsafe impl Sync for SchedulerCell {} // single-core assumption
static SCHED: SchedulerCell = SchedulerCell(UnsafeCell::new(Scheduler::new()));

// Safety: single-core assumption; external synchronization via disabling interrupts.
#[inline(always)]
fn with<F, R>(f: F) -> R where F: FnOnce(&mut Scheduler) -> R {
    // In a real system, wrap with arch::disable_interrupts()/enable_interrupts().
    // For now we call stubs to illustrate critical section.
    crate::arch::disable_interrupts();
    let r = unsafe { f(&mut *SCHED.0.get()) };
    crate::arch::enable_interrupts();
    r
}

// Public API wrappers
pub fn add_task(task: Task) -> Result<usize, ()> { with(|s| s.add_task(task)) }
pub fn post_event(events: u32) { with(|s| s.post_event(events)); }
pub fn block_current(events: u32) { with(|s| s.block_current_on(events)); }
pub fn schedule() -> Option<Task> { with(|s| s.schedule().cloned()) }
#[allow(dead_code)]
pub fn current_task() -> Option<Task> { with(|s| s.current_task().cloned()) }

// Called from an interrupt context to signal an event (must be fast / minimal work).
#[allow(dead_code)]
pub fn interrupt_event(events: u32) {
    // Minimal critical section: just wake tasks.
    with(|s| s.post_event(events));
}

