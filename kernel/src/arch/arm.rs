//! ============================================================================
//! MODULE : arch::arm — ARM Cortex-M architecture layer
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   ARM-specific implementation of the arch contracts: exception plumbing
//!   (cortex-m-rt), the SysTick 1 kHz tick source, and the early console
//!   (pre-driver UART writes to PL011 UART0).
//!
//! ROLE IN BOOT FLOW
//!   cortex-m-rt Reset -> main() -> kernel::init() -> ArmArch::init +
//!   init_tick() (this module arms SysTick) -> scheduler loop; every SysTick
//!   overflow calls tick_isr() (one atomic add).
//!
//! MEMORY BUDGET
//!   Handlers are tiny; SysTick registers are core-private (no driver RAM).
//!
//! TICK MATH
//!   LM3S6965 core clock = 12 MHz (QEMU model), tick = 1 kHz:
//!   RELOAD = 12_000_000 / 1_000 - 1 = 11_999.
//!   CSR = ENABLE | TICKINT | CLKSOURCE (processor clock).
//! ============================================================================

use crate::arch::{ArchInit, MemoryLayout};

/// Core clock of the LM3S6965 QEMU model (Hz).
pub const CORE_CLOCK_HZ: u32 = 12_000_000;

// ---------------------------------------------------------------------------
// SysTick tick source
// ---------------------------------------------------------------------------
/// SysTick registers (core-private, at 0xE000_E010).
mod systick {
    pub const BASE: usize = 0xE000_E010;
    pub const CSR: usize = 0x00; // Control and status
    pub const RVR: usize = 0x04; // Reload value
    pub const CVR: usize = 0x08; // Current value

    pub const CSR_ENABLE: u32 = 1 << 0; // Counter on
    pub const CSR_TICKINT: u32 = 1 << 1; // Raise SysTick exception
    pub const CSR_CLKSOURCE: u32 = 1 << 2; // Clock = processor clock
}

/// Program SysTick for a 1 kHz tick and enable its interrupt.
pub fn init_tick() {
    let reload = (CORE_CLOCK_HZ / crate::arch::TICK_RATE_HZ) - 1;

    // SAFETY: SysTick registers are core-private MMIO, always present on
    // Cortex-M3.
    unsafe {
        core::ptr::write_volatile((systick::BASE + systick::RVR) as *mut u32, reload);
        core::ptr::write_volatile((systick::BASE + systick::CVR) as *mut u32, 0); // force reload
        core::ptr::write_volatile(
            (systick::BASE + systick::CSR) as *mut u32,
            systick::CSR_ENABLE | systick::CSR_TICKINT | systick::CSR_CLKSOURCE,
        );
    }
}

// Exception handlers for ARM Cortex-M
#[cfg(target_arch = "arm")]
use cortex_m_rt::{exception};

/// Pre-init function called before main memory initialization
#[no_mangle]
pub unsafe extern "C" fn __pre_init() {
    // Nothing to do for basic setup
}

/// Default handler for unhandled interrupts
#[no_mangle]
pub unsafe extern "C" fn DefaultHandler() {
    loop {
        cortex_m::asm::wfi();
    }
}

// Exception handlers - cortex-m-rt requires these to be defined
#[exception]
unsafe fn NonMaskableInt() {
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn MemoryManagement() {
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn BusFault() {
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn UsageFault() {
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn SVCall() {
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn DebugMonitor() {
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn PendSV() {
    loop {
        cortex_m::asm::wfi();
    }
}

/// SysTick exception — the 1 kHz kernel tick.
///
/// ISR-safety contract: bumps the arch tick latch (one atomic add) and
/// returns. It must NEVER touch scheduler state: the scheduler holds `&mut`
/// on its globals from the main context, and critical sections here would
/// nest incorrectly.
#[exception]
unsafe fn SysTick() {
    crate::arch::tick_isr();
}

// Hard fault handler
#[exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    // Print fault information via semihosting for debugging
    use cortex_m_semihosting::hprintln;
    let _ = hprintln!("Hard Fault at 0x{:x}", ef.pc());
    let _ = hprintln!("R0: 0x{:x}, R1: 0x{:x}, R2: 0x{:x}, R3: 0x{:x}", 
                     ef.r0(), ef.r1(), ef.r2(), ef.r3());
    
    loop {
        // Infinite loop on hard fault
        cortex_m::asm::wfi();
    }
}

/// ARM architecture implementation
pub struct ArmArch;

impl ArchInit for ArmArch {
    fn init() {
        // NOTE: UART bring-up deliberately moved OUT of arch init — the
        // Pl011Uart driver owns it now (see drivers::init_platform_devices).
        Self::irq_init();
        Self::setup_memory_protection();
    }
    
    fn irq_init() {
        // Initialize interrupts for ARM
        // SysTick is armed later by arch::init_tick() (kernel::init step 4);
        // external IRQs stay masked until the NVIC driver lands (Phase 2).
    }
    
    fn setup_memory_protection() {
        // Set up MPU if available
        // For now, basic setup
    }
}

/// ARM-specific memory layout implementation
#[allow(dead_code)]
pub struct ArmMemoryLayout;

impl MemoryLayout for ArmMemoryLayout {
    fn ram_start() -> usize {
        0x20000000 // Standard ARM Cortex-M RAM start
    }

    fn ram_size() -> usize {
        64 * 1024 // 64KB RAM for LM3S6965
    }

    fn flash_start() -> usize {
        0x00000000 // Flash start
    }

    fn flash_size() -> usize {
        256 * 1024 // 256KB Flash for LM3S6965
    }

    fn stack_top() -> usize {
        Self::ram_start() + Self::ram_size()
    }

    fn heap_start() -> usize {
        Self::ram_start() + (Self::ram_size() / 2) // Middle of RAM
    }

    fn heap_size() -> usize {
        Self::ram_size() / 4 // Quarter of RAM for heap
    }
}

/// Interrupt control helpers for ARM Cortex-M (per-arch API; the arch facade
/// in arch/mod.rs uses inline asm directly, so these stay as exported surface
/// for the NVIC driver in Phase 2).
#[allow(dead_code)]
pub fn disable_interrupts() {
    unsafe {
        core::arch::asm!("cpsid i", options(nomem, nostack));
    }
}

#[allow(dead_code)]
pub fn enable_interrupts() {
    unsafe {
        core::arch::asm!("cpsie i", options(nomem, nostack));
    }
}

/// Early debug output for ARM
pub fn early_println(msg: &str) {
    // LM3S6965EVB UART0 at 0x4000C000
    const UART_BASE: usize = 0x4000C000;
    const UARTDR: usize = UART_BASE + 0x000; // Data register

    unsafe {
        for byte in msg.bytes() {
            // Write byte directly to UART data register
            // QEMU should handle the UART configuration
            core::ptr::write_volatile(UARTDR as *mut u32, byte as u32);
        }
        // Add newline
        core::ptr::write_volatile(UARTDR as *mut u32, b'\n' as u32);
    }
}

/// Yield CPU to other tasks (cooperative multitasking)
#[allow(dead_code)]
pub fn yield_cpu() {
    unsafe {
        // ARM wait for interrupt instruction
        core::arch::asm!("wfi", options(nomem, nostack));
    }
}

/// Shutdown system
#[allow(dead_code)]
pub fn shutdown() -> ! {
    // Disable interrupts and halt
    unsafe {
        core::arch::asm!("cpsid i", options(nomem, nostack));
    }
    
    loop {
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack));
        }
    }
}
