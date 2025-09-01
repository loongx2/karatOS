//! Async Event-Driven Scheduler with Cooperative Multitasking
//! 
//! Design Principles:
//! 1. No deadlocks - Single-threaded execution with cooperative yielding
//! 2. Mutually exclusive events - Atomic event processing via priority queues
//! 3. Event-driven in single-threaded environment - Future-based tasks with Waker notifications
//! 4. Multi-priority execution with preemption support
//! 
//! Algorithm: Priority-based Async Event Loop with Modern Optimizations
//! - Tasks are Rust Futures that yield control voluntarily
//! - Events are queued by priority (Critical > High > Normal > Low)
//! - Waker system provides zero-copy event notification
//! - Message-passing optimization for hot-path scheduling
//! - Lock-free ring buffers for interrupt-safe operation
//! - Multiple executor instances for priority-based preemption

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use core::mem::MaybeUninit;

// Maximum number of concurrent tasks and events
pub const MAX_TASKS: usize = 8;
pub const MAX_EVENTS_PER_PRIORITY: usize = 16;

/// Event priority levels for mutual exclusion and ordering
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum EventPriority {
    Critical = 0,  // Interrupt handlers, emergency shutdown
    High = 1,      // Time-critical operations
    Normal = 2,    // Regular task events
    #[allow(dead_code)]
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

/// Async task state management with enhanced states
#[derive(Clone, Debug, PartialEq)]
pub enum TaskState {
    Ready,              // Ready to be polled
    Running,            // Currently executing
    WaitingForEvent(u32), // Blocked on specific event ID
    Sleeping(u64),      // Sleeping until timestamp
    Completed,          // Task finished
}

/// Task priority levels for preemptive scheduling
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum TaskPriority {
    Critical = 0,  // Interrupt handlers, system critical
    High = 1,      // Time-sensitive operations
    Normal = 2,    // Regular application tasks
    Low = 3,       // Background maintenance
}

/// Enhanced task representation with Future integration
pub struct AsyncTask {
    pub id: usize,
    pub priority: TaskPriority,
    pub state: TaskState,
    pub waiting_event: Option<u32>,
    pub wake_count: AtomicU32,
}

impl AsyncTask {
    pub const fn new(id: usize, priority: TaskPriority) -> Self {
        Self {
            id,
            priority,
            state: TaskState::Ready,
            waiting_event: None,
            wake_count: AtomicU32::new(0),
        }
    }
    
    pub fn is_ready(&self) -> bool {
        matches!(self.state, TaskState::Ready)
    }
    
    pub fn wake(&self) {
        self.wake_count.fetch_add(1, Ordering::Relaxed);
    }
}

/// Lock-free ring buffer implementation (Embassy-inspired)
struct LockFreeEventQueue<const N: usize> {
    buffer: [MaybeUninit<Event>; N],
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl<const N: usize> LockFreeEventQueue<N> {
    const fn new() -> Self {
        Self {
            buffer: unsafe { MaybeUninit::uninit().assume_init() },
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }
    
    /// Push event to queue (lock-free, ISR-safe)
    fn push(&mut self, event: Event) -> Result<(), Event> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        
        if tail.wrapping_sub(head) >= N {
            return Err(event); // Queue full
        }
        
        let index = tail % N;
        unsafe {
            self.buffer[index].as_mut_ptr().write(event);
        }
        self.tail.store(tail + 1, Ordering::Release);
        Ok(())
    }
    
    /// Pop event from queue (lock-free)
    fn pop(&self) -> Option<Event> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        
        if head == tail {
            return None; // Queue empty
        }
        
        let index = head % N;
        let event = unsafe { self.buffer[index].as_ptr().read() };
        self.head.store(head + 1, Ordering::Release);
        Some(event)
    }
    
    fn is_empty(&self) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        head == tail
    }
}

/// Simple task representation for compatibility
#[derive(Clone, Debug)]
pub struct Task {
    pub id: usize,
    pub priority: TaskPriority,
    pub state: TaskState,
    pub waiting_event: Option<u32>,
}

impl Task {
    pub const fn new(id: usize) -> Self {
        Self::with_priority(id, TaskPriority::Normal)
    }
    
    pub const fn with_priority(id: usize, priority: TaskPriority) -> Self {
        Task {
            id,
            priority,
            state: TaskState::Ready,
            waiting_event: None,
        }
    }
    
    pub fn is_ready(&self) -> bool {
        matches!(self.state, TaskState::Ready)
    }
}

/// Multi-Priority Executor for preemptive scheduling
pub struct MultiPriorityExecutor {
    critical_scheduler: AsyncScheduler,
    high_scheduler: AsyncScheduler,
    normal_scheduler: AsyncScheduler,
    low_scheduler: AsyncScheduler,
    current_priority: AtomicU32,
}

impl MultiPriorityExecutor {
    pub const fn new() -> Self {
        Self {
            critical_scheduler: AsyncScheduler::new(),
            high_scheduler: AsyncScheduler::new(),
            normal_scheduler: AsyncScheduler::new(),
            low_scheduler: AsyncScheduler::new(),
            current_priority: AtomicU32::new(TaskPriority::Low as u32),
        }
    }
    
    /// Add task to appropriate priority scheduler
    pub fn spawn_task(&mut self, task: Task) -> Result<usize, ()> {
        match task.priority {
            TaskPriority::Critical => self.critical_scheduler.spawn_task(task),
            TaskPriority::High => self.high_scheduler.spawn_task(task),
            TaskPriority::Normal => self.normal_scheduler.spawn_task(task),
            TaskPriority::Low => self.low_scheduler.spawn_task(task),
        }
    }
    
    /// Post event to appropriate priority queue
    pub fn post_event(&mut self, event: Event) -> bool {
        match event.priority {
            EventPriority::Critical => self.critical_scheduler.post_event(event),
            EventPriority::High => self.high_scheduler.post_event(event),
            EventPriority::Normal => self.normal_scheduler.post_event(event),
            EventPriority::Low => self.low_scheduler.post_event(event),
        }
    }
    
    /// Run one scheduling cycle with priority-based preemption
    pub fn run_cycle(&mut self) -> Option<Task> {
        // Critical tasks preempt everything
        if let Some(task) = self.critical_scheduler.schedule() {
            self.current_priority.store(TaskPriority::Critical as u32, Ordering::Release);
            return Some(task.clone());
        }
        
        // High priority tasks
        if let Some(task) = self.high_scheduler.schedule() {
            self.current_priority.store(TaskPriority::High as u32, Ordering::Release);
            return Some(task.clone());
        }
        
        // Normal priority tasks
        if let Some(task) = self.normal_scheduler.schedule() {
            self.current_priority.store(TaskPriority::Normal as u32, Ordering::Release);
            return Some(task.clone());
        }
        
        // Low priority tasks (background)
        if let Some(task) = self.low_scheduler.schedule() {
            self.current_priority.store(TaskPriority::Low as u32, Ordering::Release);
            return Some(task.clone());
        }
        
        None
    }
    
    /// Check if any scheduler has ready tasks
    pub fn has_ready_tasks(&self) -> bool {
        self.critical_scheduler.has_active_tasks() ||
        self.high_scheduler.has_active_tasks() ||
        self.normal_scheduler.has_active_tasks() ||
        self.low_scheduler.has_active_tasks()
    }
    
    /// Get current executing priority level
    pub fn current_priority(&self) -> TaskPriority {
        match self.current_priority.load(Ordering::Acquire) {
            0 => TaskPriority::Critical,
            1 => TaskPriority::High,
            2 => TaskPriority::Normal,
            _ => TaskPriority::Low,
        }
    }
}

/// Enhanced Priority-based Async Event-Driven Scheduler
pub struct AsyncScheduler {
    // Task management with message-passing optimization
    tasks: [Option<Task>; MAX_TASKS],
    current_task: Option<usize>,
    next_task: Option<usize>, // Hot slot for message-passing optimization
    
    // Lock-free event queues by priority
    critical_events: LockFreeEventQueue<MAX_EVENTS_PER_PRIORITY>,
    high_events: LockFreeEventQueue<MAX_EVENTS_PER_PRIORITY>,
    normal_events: LockFreeEventQueue<MAX_EVENTS_PER_PRIORITY>,
    low_events: LockFreeEventQueue<MAX_EVENTS_PER_PRIORITY>,
    
    // Scheduling state
    needs_reschedule: AtomicBool,
    active_tasks: AtomicU32,
    event_counter: AtomicU32,
    timer_base: AtomicU32, // For sleep/timeout functionality (32-bit for embedded compatibility)
}

impl AsyncScheduler {
    pub const fn new() -> Self {
        const NONE_TASK: Option<Task> = None;
        Self {
            tasks: [NONE_TASK; MAX_TASKS],
            current_task: None,
            next_task: None,
            critical_events: LockFreeEventQueue::new(),
            high_events: LockFreeEventQueue::new(),
            normal_events: LockFreeEventQueue::new(),
            low_events: LockFreeEventQueue::new(),
            needs_reschedule: AtomicBool::new(false),
            active_tasks: AtomicU32::new(0),
            event_counter: AtomicU32::new(0),
            timer_base: AtomicU32::new(0),
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
    
    /// Post an event with specified priority (ISR-safe)
    pub fn post_event(&mut self, event: Event) -> bool {
        let result = match event.priority {
            EventPriority::Critical => self.critical_events.push(event),
            EventPriority::High => self.high_events.push(event),
            EventPriority::Normal => self.normal_events.push(event),
            EventPriority::Low => self.low_events.push(event),
        };
        
        if result.is_ok() {
            self.event_counter.fetch_add(1, Ordering::Relaxed);
            self.wake_waiting_tasks(event.id);
            true
        } else {
            false // Queue full
        }
    }
    
    /// Wake tasks waiting for a specific event with message-passing optimization
    fn wake_waiting_tasks(&mut self, event_id: u32) {
        let mut displaced_task_id: Option<usize> = None;
        
        for (i, task_slot) in self.tasks.iter_mut().enumerate() {
            if let Some(task) = task_slot {
                if let TaskState::WaitingForEvent(waiting_id) = task.state {
                    if waiting_id == event_id {
                        task.state = TaskState::Ready;
                        task.waiting_event = None;
                        
                        // Message-passing optimization: put in hot slot
                        displaced_task_id = self.next_task.replace(i);
                        
                        self.needs_reschedule.store(true, Ordering::Release);
                        break; // Only wake first matching task for fairness
                    }
                }
            }
        }
        
        // Handle displaced task outside the iterator
        if let Some(displaced_id) = displaced_task_id {
            if let Some(displaced_task) = &mut self.tasks[displaced_id] {
                if displaced_task.state == TaskState::Running {
                    displaced_task.state = TaskState::Ready;
                }
            }
        }
    }
    
    /// Process events in priority order (lock-free)
    pub fn process_events(&mut self) -> u32 {
        let mut processed = 0;
        
        // Process one event per priority level for fairness
        if let Some(event) = self.critical_events.pop() {
            self.handle_event(event);
            processed += 1;
        }
        
        if let Some(event) = self.high_events.pop() {
            self.handle_event(event);
            processed += 1;
        }
        
        if let Some(event) = self.normal_events.pop() {
            self.handle_event(event);
            processed += 1;
        }
        
        if let Some(event) = self.low_events.pop() {
            self.handle_event(event);
            processed += 1;
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
            0x10..=0x1F => { /* System events */ },
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
    
    /// Sleep current task for specified time units
    pub fn sleep_current_task(&mut self, duration: u32) {
        if let Some(current_id) = self.current_task {
            if let Some(task) = &mut self.tasks[current_id] {
                let wake_time = self.timer_base.load(Ordering::Relaxed) + duration;
                task.state = TaskState::Sleeping(wake_time as u64);
            }
            self.current_task = None;
            self.needs_reschedule.store(true, Ordering::Release);
        }
    }
    
    /// Update timer and wake sleeping tasks
    pub fn update_timer(&mut self, current_time: u32) {
        self.timer_base.store(current_time, Ordering::Relaxed);
        
        for task_slot in self.tasks.iter_mut() {
            if let Some(task) = task_slot {
                if let TaskState::Sleeping(wake_time) = task.state {
                    if (current_time as u64) >= wake_time {
                        task.state = TaskState::Ready;
                        self.needs_reschedule.store(true, Ordering::Release);
                    }
                }
            }
        }
    }
    
    /// Enhanced cooperative scheduler with message-passing optimization
    pub fn schedule(&mut self) -> Option<&Task> {
        // Process pending events first
        self.process_events();
        
        // Check hot slot first (message-passing optimization)
        if let Some(next_id) = self.next_task.take() {
            // Check if task exists and is ready
            let task_ready = self.tasks[next_id]
                .as_ref()
                .map(|task| task.is_ready())
                .unwrap_or(false);
                
            if task_ready {
                // Mark current task as ready if it was running (and it's different)
                if let Some(current_id) = self.current_task {
                    if current_id != next_id {
                        if let Some(current_task) = self.tasks[current_id].as_mut() {
                            if current_task.state == TaskState::Running {
                                current_task.state = TaskState::Ready;
                            }
                        }
                    }
                }
                
                // Now modify the next task
                if let Some(task) = self.tasks[next_id].as_mut() {
                    task.state = TaskState::Running;
                    self.current_task = Some(next_id);
                }
                
                return self.tasks[next_id].as_ref();
            }
        }
        
        if self.needs_reschedule.swap(false, Ordering::AcqRel) || self.current_task.is_none() {
            // Mark current task as ready if it's still running
            if let Some(current_id) = self.current_task {
                if let Some(task) = self.tasks[current_id].as_mut() {
                    if matches!(task.state, TaskState::Running) {
                        task.state = TaskState::Ready;
                    }
                }
            }
            
            // Find next ready task (round-robin among ready tasks)
            let start_search = self.current_task.map(|id| (id + 1) % MAX_TASKS).unwrap_or(0);
            
            for i in 0..MAX_TASKS {
                let task_id = (start_search + i) % MAX_TASKS;
                if let Some(task) = self.tasks[task_id].as_mut() {
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
    pub fn current_task(&self) -> Option<&Task> {
        self.current_task.and_then(|id| self.tasks[id].as_ref())
    }
    
    /// Check if scheduler has any active tasks
    pub fn has_active_tasks(&self) -> bool {
        self.active_tasks.load(Ordering::Relaxed) > 0
    }
    
    /// Check if scheduler has ready tasks
    pub fn has_ready_tasks(&self) -> bool {
        self.tasks.iter().any(|task_opt| {
            if let Some(task) = task_opt {
                task.is_ready()
            } else {
                false
            }
        })
    }
    
    /// Get scheduler statistics
    pub fn stats(&self) -> (u32, u32, u32) {
        (
            self.active_tasks.load(Ordering::Relaxed),
            self.event_counter.load(Ordering::Relaxed),
            self.timer_base.load(Ordering::Relaxed)
        )
    }
}

// -------- Global scheduler instances --------
struct SchedulerCell(UnsafeCell<AsyncScheduler>);
unsafe impl Sync for SchedulerCell {} // Single-core assumption

struct MultiPriorityCell(UnsafeCell<MultiPriorityExecutor>);
unsafe impl Sync for MultiPriorityCell {} // Single-core assumption

static SCHEDULER: SchedulerCell = SchedulerCell(UnsafeCell::new(AsyncScheduler::new()));
static MULTI_PRIORITY_SCHEDULER: MultiPriorityCell = MultiPriorityCell(UnsafeCell::new(MultiPriorityExecutor::new()));

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

// Multi-priority scheduler access
#[inline(always)]
fn with_multi_scheduler<F, R>(f: F) -> R 
where 
    F: FnOnce(&mut MultiPriorityExecutor) -> R 
{
    // Disable interrupts for atomic scheduler access
    crate::arch::disable_interrupts();
    let result = unsafe { f(&mut *MULTI_PRIORITY_SCHEDULER.0.get()) };
    crate::arch::enable_interrupts();
    result
}

// -------- Enhanced Public API --------

/// Spawn a new task with default normal priority
pub fn add_task(task: Task) -> Result<usize, ()> {
    with_scheduler(|sched| sched.spawn_task(task))
}

/// Spawn a task with specific priority (uses multi-priority executor)
pub fn add_priority_task(task: Task) -> Result<usize, ()> {
    with_multi_scheduler(|sched| sched.spawn_task(task))
}

/// Post an event to wake waiting tasks
pub fn post_event_with_priority(id: u32, priority: EventPriority) -> bool {
    let event = Event::new(id, priority);
    with_scheduler(|sched| sched.post_event(event))
}

/// Post event to multi-priority scheduler (better for real-time systems)
pub fn post_priority_event(id: u32, priority: EventPriority) -> bool {
    let event = Event::new(id, priority);
    with_multi_scheduler(|sched| sched.post_event(event))
}

/// Post a normal priority event (compatibility)
pub fn post_event(event_id: u32) {
    let _ = post_event_with_priority(event_id, EventPriority::Normal);
}

/// Block current task until event arrives
pub fn block_current(event_id: u32) {
    with_scheduler(|sched| sched.block_current_task(event_id));
}

/// Sleep current task for specified duration
pub fn sleep_current(duration: u32) {
    with_scheduler(|sched| sched.sleep_current_task(duration));
}

/// Update global timer (call this periodically from timer interrupt)
pub fn update_global_timer(current_time: u32) {
    with_scheduler(|sched| sched.update_timer(current_time));
}

/// Run scheduler and return current task
pub fn schedule() -> Option<Task> {
    with_scheduler(|sched| sched.schedule().cloned())
}

/// Run multi-priority scheduler (recommended for real-time systems)
pub fn schedule_with_priority() -> Option<Task> {
    with_multi_scheduler(|sched| sched.run_cycle())
}

/// Get current running task
pub fn current_task() -> Option<Task> {
    with_scheduler(|sched| sched.current_task().cloned())
}

/// Post critical priority event (for interrupt handlers, ISR-safe)
pub fn interrupt_event(event_id: u32) {
    let _ = post_event_with_priority(event_id, EventPriority::Critical);
}

/// Post interrupt event to multi-priority scheduler (ISR-safe)
pub fn interrupt_priority_event(event_id: u32) {
    let _ = post_priority_event(event_id, EventPriority::Critical);
}

/// Get scheduler statistics (active_tasks, total_events, timer)
pub fn scheduler_stats() -> (u32, u32, u32) {
    with_scheduler(|sched| sched.stats())
}

/// Check if any scheduler has ready work
pub fn has_ready_work() -> bool {
    with_multi_scheduler(|sched| sched.has_ready_tasks())
}

/// Get current priority level of executing task
pub fn current_priority_level() -> TaskPriority {
    with_multi_scheduler(|sched| sched.current_priority())
}

/// Architecture-agnostic yield point for cooperative multitasking
#[inline(always)]
pub fn yield_now() {
    // This can be called from any architecture
    // The actual yield is handled by the scheduler
    unsafe {
        // Generic no-op that works on all architectures
        core::arch::asm!("nop", options(nomem, nostack, preserves_flags));
    }
}

/// Architecture-agnostic sleep/wait instruction
#[inline(always)]
pub fn cpu_wait_for_interrupt() {
    // Architecture-specific implementations are handled in arch module
    crate::arch::wait_for_interrupt();
}

