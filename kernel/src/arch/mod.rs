//! ============================================================================
//! MODULE : arch — architecture abstraction layer (ARM ⇄ RISC-V)
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   The ONLY module allowed to contain `target_arch` conditionals for kernel
//!   services. Everything above this layer (kernel, drivers, scheduler,
//!   future modules) stays architecture-agnostic.
//!
//! ROLE IN BOOT FLOW
//!   boot entry -> early_println (pre-driver console) -> kernel::init() ->
//!   ArchInit::init + init_tick (this module) -> scheduler runs, draining
//!   ticks via advance_time().
//!
//! TIME MODEL (Phase 0 tick base)
//!   * Hardware tick (SysTick / CLINT) fires at TICK_RATE_HZ.
//!   * The ISR calls tick_isr(): ONE atomic add on TICK_LATCH. It never
//!     touches scheduler state — this is what makes the tick ISR-safe while
//!     the cooperative scheduler keeps `&mut` on its globals.
//!   * Main loop calls advance_time(): drains the latch into MILLIS and lets
//!     the caller feed the scheduler's timer. Monotonic, ms-resolution.
//!
//! OOP MODEL
//!   `MemoryLayout` / `ArchInit` / `Architecture` traits describe per-arch
//!   capabilities; `ArmArch` and `RiscvArch` are the implementations.
//!
//! MEMORY BUDGET
//!   Two AtomicU32 (latch + millis) in .bss; everything else is code.
//! ============================================================================

use core::sync::atomic::{AtomicBool, Ordering};

// Interrupt state for critical sections
static INTERRUPTS_ENABLED: AtomicBool = AtomicBool::new(true);

// ---------------------------------------------------------------------------
// Tick time base (ISR-safe latch + monotonic millis)
//
// Implemented WITHOUT core atomics on purpose: the riscv32imc target (no A
// extension) has no atomic RMW instructions, and this path must compile for
// every karatOS target. Portability contract instead:
//   * tick_isr() runs ONLY from the arch timer ISR — single-writer context.
//   * advance_time() guards the drain with disable/enable_interrupts, so the
//     ISR can never observe a torn drain.
// ---------------------------------------------------------------------------

/// Kernel tick rate. 1 kHz gives ms-resolution sleeps at negligible ISR load.
pub const TICK_RATE_HZ: u32 = 1000;

/// Ticks raised by the ISR, drained by `advance_time()` from the main loop.
static mut TICK_LATCH: u32 = 0;

/// Milliseconds since boot (monotonic, may wrap at 2^32 ms ≈ 49.7 days).
static mut MILLIS: u32 = 0;

/// Tick ISR hook — called from arch timer handlers ONLY.
/// Deliberately minimal (one load-add-store) so ISR latency stays constant.
#[inline(always)]
pub fn tick_isr() {
    // SAFETY: single-writer (ISR context at one priority level); no other
    // code writes TICK_LATCH outside an interrupts-disabled critical section.
    unsafe {
        TICK_LATCH = TICK_LATCH.wrapping_add(1);
    }
}

/// Drain pending ticks into MILLIS; returns the new millis value.
/// Call once per scheduler cycle from the main loop.
pub fn advance_time() -> u32 {
    disable_interrupts();
    // SAFETY: guarded by the critical section above (ISR cannot interleave).
    let pending = unsafe { TICK_LATCH };
    unsafe { TICK_LATCH = 0 };
    enable_interrupts();

    if pending != 0 {
        disable_interrupts();
        // SAFETY: MILLIS is only mutated inside interrupts-disabled sections.
        unsafe {
            MILLIS = MILLIS.wrapping_add(pending);
        }
        enable_interrupts();
    }

    millis_since_boot()
}

/// Milliseconds since boot (monotonic). Safe from any context: a torn u32
/// read is impossible on 32-bit targets, and a stale value is harmless.
#[inline(always)]
pub fn millis_since_boot() -> u32 {
    // SAFETY: aligned 32-bit read; at worst returns a value one tick old.
    unsafe { MILLIS }
}

/// Start the architecture tick source (SysTick on ARM, CLINT on RISC-V).
/// Must run before the scheduler loop expects time to advance.
pub fn init_tick() {
    #[cfg(target_arch = "arm")]
    arm::init_tick();

    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    riscv::init_tick();

    #[cfg(not(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64")))]
    {
        // Host: no hardware tick; time stays frozen (fine for unit tests).
    }
}

// Import architecture-specific modules
#[cfg(any(feature = "arm", target_arch = "arm"))]
pub mod arm;

#[cfg(any(feature = "riscv", any(target_arch = "riscv32", target_arch = "riscv64")))]
pub mod riscv;

/// Memory layout trait for architecture-specific configurations
#[allow(dead_code)]
pub trait MemoryLayout {
    fn ram_start() -> usize;
    fn ram_size() -> usize;
    fn flash_start() -> usize;
    fn flash_size() -> usize;
    fn stack_top() -> usize;
    fn heap_start() -> usize;
    fn heap_size() -> usize;
}

/// Architecture initialization trait
#[allow(dead_code)]
pub trait ArchInit {
    fn init();
    fn irq_init();
    fn setup_memory_protection();
}

/// Architecture abstraction trait
#[allow(dead_code)]
pub trait Architecture {
    type MemoryLayout: MemoryLayout;
    type Init: ArchInit;
}

/// Early println for debugging (before full system init)
#[allow(dead_code)]
pub fn early_println(msg: &str) {
    #[cfg(target_arch = "arm")]
    arm::early_println(msg);

    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    riscv::early_println(msg);

    // Host builds: `no_std` has no println!, so output is dropped — host
    // tests exercise logic, not I/O. Kept `no_std`-clean on purpose.
    #[cfg(not(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64")))]
    {
        let _ = msg;
    }
}

/// Disable interrupts for critical sections
#[allow(dead_code)]
pub fn disable_interrupts() {
    INTERRUPTS_ENABLED.store(false, Ordering::SeqCst);
    
    #[cfg(target_arch = "arm")]
    unsafe {
        core::arch::asm!("cpsid i");
    }
    
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    unsafe {
        core::arch::asm!("csrci mstatus, 8");
    }
}

/// Enable interrupts after critical sections
#[allow(dead_code)]
pub fn enable_interrupts() {
    INTERRUPTS_ENABLED.store(true, Ordering::SeqCst);
    
    #[cfg(target_arch = "arm")]
    unsafe {
        core::arch::asm!("cpsie i");
    }
    
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    unsafe {
        core::arch::asm!("csrsi mstatus, 8");
    }
}

/// Yield CPU to other tasks (cooperative multitasking)
#[allow(dead_code)]
pub fn arch_yield() {
    #[cfg(target_arch = "arm")]
    arm::yield_cpu();
    
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    riscv::yield_cpu();
}

/// Architecture-agnostic wait for interrupt
#[allow(dead_code)]
pub fn wait_for_interrupt() {
    #[cfg(target_arch = "arm")]
    unsafe {
        // ARM WFE (Wait For Event) - more efficient than WFI for our scheduler
        core::arch::asm!("wfe");
    }
    
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    unsafe {
        // RISC-V WFI (Wait For Interrupt)
        core::arch::asm!("wfi");
    }
    
    #[cfg(not(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64")))]
    {
        // Host platform - do nothing (for testing)
    }
}

/// Get current interrupt state
    #[allow(dead_code)]
pub fn interrupts_enabled() -> bool {
    INTERRUPTS_ENABLED.load(Ordering::SeqCst)
}

/// Architecture-specific shutdown
#[allow(dead_code)]
pub fn arch_shutdown() -> ! {
    disable_interrupts();
    
    #[cfg(target_arch = "arm")]
    arm::shutdown();
    
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    riscv::shutdown();
    
    #[cfg(not(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64")))]
    loop {
        core::hint::spin_loop();
    }
}
