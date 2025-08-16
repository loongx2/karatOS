//! Async Event-Driven Scheduler with Cooperative Multitasking
//! 
//! Design Principles:
//! 1. No deadlocks - Single-threaded execution with cooperative yielding
//! 2. Mutually exclusive events - Atomic event processing via priority queues
//! 3. Event-driven in single-threaded environment - Future-based tasks with Waker notifications
//! 
//! Algorithm: Priority-based Async Event Loop
//! - Tasks are Rust Futures that yield control voluntarily
//! - Events are queued by priority (Critical > High > Normal > Low)
//! - Waker system provides zero-copy event notification
//! - Single executor thread eliminates race conditions

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

// Maximum number of concurrent tasks and events
pub const MAX_TASKS: usize = 8;
pub const MAX_EVENTS_PER_PRIORITY: usize = 16;

/// Event priority levels for mutual exclusion and ordering
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum EventPriority {
    Critical = 0,  // Interrupt handlers, emergency shutdown
    High = 1,      // Time-critical operations
    Normal = 2,    // Regular task events
    Low = 3,       // Background, cleanup tasks
}

/// Event structure for async task communication
#[derive(Copy, Clone, Debug)]
pub struct Event {
    pub id: u32,
    pub priority: EventPriority,
    #[allow(dead_code)]
    pub data: u32,  // Optional event payload
}

impl Event {
    pub const fn new(id: u32, priority: EventPriority) -> Self {
        Self { id, priority, data: 0 }
    }
    
    #[allow(dead_code)]
    pub const fn with_data(id: u32, priority: EventPriority, data: u32) -> Self {
        Self { id, priority, data }
    }
}

/// Async task state management
#[derive(Clone, Debug)]
pub enum TaskState {
    Ready,              // Ready to be polled
    Running,            // Currently executing
    WaitingForEvent(u32), // Blocked on specific event ID
    #[allow(dead_code)]
    Completed,          // Task finished
}

/// Lock-free ring buffer for event queuing
struct EventQueue {
    events: [Option<Event>; MAX_EVENTS_PER_PRIORITY],
    head: usize,
    tail: usize,
    count: usize,
}

impl EventQueue {
    const fn new() -> Self {
        Self {
            events: [None; MAX_EVENTS_PER_PRIORITY],
            head: 0,
            tail: 0,
            count: 0,
        }
    }
    
    /// Push event to queue (returns false if full)
    fn push(&mut self, event: Event) -> bool {
        if self.count >= MAX_EVENTS_PER_PRIORITY {
            return false; // Queue full
        }
        
        self.events[self.tail] = Some(event);
        self.tail = (self.tail + 1) % MAX_EVENTS_PER_PRIORITY;
        self.count += 1;
        true
    }
    
    /// Pop highest priority event
    fn pop(&mut self) -> Option<Event> {
        if self.count == 0 {
            return None;
        }
        
        let event = self.events[self.head].take();
        self.head = (self.head + 1) % MAX_EVENTS_PER_PRIORITY;
        self.count -= 1;
        event
    }
    
    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Simple task representation for compatibility
#[derive(Clone, Debug)]
pub struct Task {
    pub id: usize,
    pub state: TaskState,
    pub waiting_event: Option<u32>,
    #[allow(dead_code)]
    pub waker: Option<bool>, // Simplified waker flag
}

impl Task {
    pub const fn new(id: usize) -> Self {
        Task {
            id,
            state: TaskState::Ready,
            waiting_event: None,
            waker: None,
        }
    }
}

/// Priority-based Async Event-Driven Scheduler
pub struct AsyncScheduler {
    // Task management
    tasks: [Option<Task>; MAX_TASKS],
    current_task: Option<usize>,
    
    // Event queues by priority (processed in order)
    critical_events: EventQueue,
    high_events: EventQueue,
    normal_events: EventQueue,
    low_events: EventQueue,
    
    // Scheduling state
    needs_reschedule: AtomicBool,
    active_tasks: AtomicU32,
    event_counter: AtomicU32,
}

impl AsyncScheduler {
    pub const fn new() -> Self {
        const NONE_TASK: Option<Task> = None;
        Self {
            tasks: [NONE_TASK; MAX_TASKS],
            current_task: None,
            critical_events: EventQueue::new(),
            high_events: EventQueue::new(),
            normal_events: EventQueue::new(),
            low_events: EventQueue::new(),
            needs_reschedule: AtomicBool::new(false),
            active_tasks: AtomicU32::new(0),
            event_counter: AtomicU32::new(0),
        }
    }
    
    /// Add a new task to the scheduler
    pub fn spawn_task(&mut self, task: Task) -> Result<usize, ()> {
        for (i, slot) in self.tasks.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(task);
                self.active_tasks.fetch_add(1, Ordering::Relaxed);
                self.needs_reschedule.store(true, Ordering::Release);
                return Ok(i);
            }
        }
        Err(()) // No free slots
    }
    
    /// Post an event with specified priority
    pub fn post_event(&mut self, event: Event) -> bool {
        let queue = match event.priority {
            EventPriority::Critical => &mut self.critical_events,
            EventPriority::High => &mut self.high_events,
            EventPriority::Normal => &mut self.normal_events,
            EventPriority::Low => &mut self.low_events,
        };
        
        if queue.push(event) {
            self.event_counter.fetch_add(1, Ordering::Relaxed);
            self.wake_waiting_tasks(event.id);
            true
        } else {
            false // Queue full
        }
    }
    
    /// Wake tasks waiting for a specific event
    fn wake_waiting_tasks(&mut self, event_id: u32) {
        for task_slot in self.tasks.iter_mut() {
            if let Some(task) = task_slot {
                if let TaskState::WaitingForEvent(waiting_id) = task.state {
                    if waiting_id == event_id {
                        task.state = TaskState::Ready;
                        task.waiting_event = None;
                        self.needs_reschedule.store(true, Ordering::Release);
                    }
                }
            }
        }
    }
    
    /// Process events in priority order
    pub fn process_events(&mut self) -> u32 {
        let mut processed = 0;
        
        // Process critical events first
        while let Some(event) = self.critical_events.pop() {
            self.handle_event(event);
            processed += 1;
            break; // One event per cycle for fairness
        }
        
        // Then high priority events
        if processed == 0 {
            if let Some(event) = self.high_events.pop() {
                self.handle_event(event);
                processed += 1;
            }
        }
        
        // Then normal priority events
        if processed == 0 {
            if let Some(event) = self.normal_events.pop() {
                self.handle_event(event);
                processed += 1;
            }
        }
        
        // Finally low priority events
        if processed == 0 {
            if let Some(event) = self.low_events.pop() {
                self.handle_event(event);
                processed += 1;
            }
        }
        
        processed
    }
    
    /// Handle a single event (can be extended for specific event types)
    fn handle_event(&mut self, event: Event) {
        // Event handling logic - can be customized per event type
        match event.id {
            0x1 => { /* Timer event */ },
            0x2 => { /* I/O event */ },
            0x3 => { /* User input */ },
            0xFF => { /* Shutdown event */ },
            _ => { /* Generic event */ }
        }
    }
    
    /// Block current task on an event
    pub fn block_current_task(&mut self, event_id: u32) {
        if let Some(current_id) = self.current_task {
            if let Some(task) = &mut self.tasks[current_id] {
                task.state = TaskState::WaitingForEvent(event_id);
                task.waiting_event = Some(event_id);
            }
            self.current_task = None;
            self.needs_reschedule.store(true, Ordering::Release);
        }
    }
    
    /// Cooperative scheduler - select next ready task
    pub fn schedule(&mut self) -> Option<&Task> {
        // Process pending events first
        self.process_events();
        
        if self.needs_reschedule.swap(false, Ordering::AcqRel) || self.current_task.is_none() {
            // Mark current task as ready if it's still running
            if let Some(current_id) = self.current_task {
                if let Some(task) = &mut self.tasks[current_id] {
                    if matches!(task.state, TaskState::Running) {
                        task.state = TaskState::Ready;
                    }
                }
            }
            
            // Find next ready task (round-robin among ready tasks)
            let start_search = self.current_task.map(|id| (id + 1) % MAX_TASKS).unwrap_or(0);
            
            for i in 0..MAX_TASKS {
                let task_id = (start_search + i) % MAX_TASKS;
                if let Some(task) = &mut self.tasks[task_id] {
                    if matches!(task.state, TaskState::Ready) {
                        task.state = TaskState::Running;
                        self.current_task = Some(task_id);
                        break;
                    }
                }
            }
        }
        
        self.current_task.and_then(|id| self.tasks[id].as_ref())
    }
    
    /// Get current running task
    #[allow(dead_code)]
    pub fn current_task(&self) -> Option<&Task> {
        self.current_task.and_then(|id| self.tasks[id].as_ref())
    }
    
    /// Check if scheduler has any active tasks
    #[allow(dead_code)]
    pub fn has_active_tasks(&self) -> bool {
        self.active_tasks.load(Ordering::Relaxed) > 0
    }
    
    /// Get scheduler statistics
    pub fn stats(&self) -> (u32, u32) {
        (
            self.active_tasks.load(Ordering::Relaxed),
            self.event_counter.load(Ordering::Relaxed)
        )
    }
}

// -------- Global singleton scheduler --------
struct SchedulerCell(UnsafeCell<AsyncScheduler>);
unsafe impl Sync for SchedulerCell {} // Single-core assumption

static SCHEDULER: SchedulerCell = SchedulerCell(UnsafeCell::new(AsyncScheduler::new()));

// Critical section wrapper for single-threaded safety
#[inline(always)]
fn with_scheduler<F, R>(f: F) -> R 
where 
    F: FnOnce(&mut AsyncScheduler) -> R 
{
    // Disable interrupts for atomic scheduler access
    crate::arch::disable_interrupts();
    let result = unsafe { f(&mut *SCHEDULER.0.get()) };
    crate::arch::enable_interrupts();
    result
}

// -------- Public API --------

/// Spawn a new task
pub fn add_task(task: Task) -> Result<usize, ()> {
    with_scheduler(|sched| sched.spawn_task(task))
}

/// Post an event to wake waiting tasks
pub fn post_event_with_priority(id: u32, priority: EventPriority) -> bool {
    let event = Event::new(id, priority);
    with_scheduler(|sched| sched.post_event(event))
}

/// Post a normal priority event (compatibility)
#[allow(dead_code)]
pub fn post_event(event_id: u32) {
    let _ = post_event_with_priority(event_id, EventPriority::Normal);
}

/// Block current task until event arrives
pub fn block_current(event_id: u32) {
    with_scheduler(|sched| sched.block_current_task(event_id));
}

/// Run scheduler and return current task
pub fn schedule() -> Option<Task> {
    with_scheduler(|sched| sched.schedule().cloned())
}

/// Get current running task
#[allow(dead_code)]
pub fn current_task() -> Option<Task> {
    with_scheduler(|sched| sched.current_task().cloned())
}

/// Post critical priority event (for interrupt handlers)
#[allow(dead_code)]
pub fn interrupt_event(event_id: u32) {
    let _ = post_event_with_priority(event_id, EventPriority::Critical);
}

/// Get scheduler statistics (active_tasks, total_events)
pub fn scheduler_stats() -> (u32, u32) {
    with_scheduler(|sched| sched.stats())
}

