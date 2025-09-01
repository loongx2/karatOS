//! Architecture abstraction layer for multi-platform support
//! Provides unified interface for ARM and RISC-V architectures

use core::sync::atomic::{AtomicBool, Ordering};

// Interrupt state for critical sections
static INTERRUPTS_ENABLED: AtomicBool = AtomicBool::new(true);

// Import architecture-specific modules
#[cfg(any(feature = "arm", target_arch = "arm"))]
pub mod arm;

#[cfg(any(feature = "riscv", target_arch = "riscv32"))]
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
    #[cfg(feature = "arm")]
    arm::early_println(msg);
    
    #[cfg(feature = "riscv")]
    riscv::early_println(msg);
    
    #[cfg(not(any(feature = "arm", feature = "riscv")))]
    {
        // Host platform - use standard output
        println!("{}", msg);
    }
}

/// Disable interrupts for critical sections
#[allow(dead_code)]
pub fn disable_interrupts() {
    INTERRUPTS_ENABLED.store(false, Ordering::SeqCst);
    
    #[cfg(feature = "arm")]
    unsafe {
        core::arch::asm!("cpsid i");
    }
    
    #[cfg(feature = "riscv")]
    unsafe {
        core::arch::asm!("csrci mstatus, 8");
    }
}

/// Enable interrupts after critical sections
#[allow(dead_code)]
pub fn enable_interrupts() {
    INTERRUPTS_ENABLED.store(true, Ordering::SeqCst);
    
    #[cfg(feature = "arm")]
    unsafe {
        core::arch::asm!("cpsie i");
    }
    
    #[cfg(feature = "riscv")]
    unsafe {
        core::arch::asm!("csrsi mstatus, 8");
    }
}

/// Yield CPU to other tasks (cooperative multitasking)
#[allow(dead_code)]
pub fn arch_yield() {
    #[cfg(feature = "arm")]
    arm::yield_cpu();
    
    #[cfg(feature = "riscv")]
    riscv::yield_cpu();
}

/// Architecture-agnostic wait for interrupt
#[allow(dead_code)]
pub fn wait_for_interrupt() {
    #[cfg(feature = "arm")]
    unsafe {
        // ARM WFE (Wait For Event) - more efficient than WFI for our scheduler
        core::arch::asm!("wfe");
    }
    
    #[cfg(feature = "riscv")]
    unsafe {
        // RISC-V WFI (Wait For Interrupt)
        core::arch::asm!("wfi");
    }
    
    #[cfg(not(any(feature = "arm", feature = "riscv")))]
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
    
    #[cfg(feature = "arm")]
    arm::shutdown();
    
    #[cfg(feature = "riscv")]
    riscv::shutdown();
    
    #[cfg(not(any(feature = "arm", feature = "riscv")))]
    loop {
        core::hint::spin_loop();
    }
}
