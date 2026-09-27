//! ============================================================================
//! MODULE : arch::riscv — RISC-V architecture layer
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   RISC-V implementation of the arch contracts: CLINT 1 kHz tick source,
//!   machine-timer trap handling (via riscv-rt's `MachineTimer` dispatch),
//!   and the early console (pre-driver NS16550A writes).
//!
//! ROLE IN BOOT FLOW
//!   riscv-rt _start -> main() -> kernel::init() -> RiscvArch::init +
//!   init_tick() (this module arms CLINT + MTIE) -> scheduler loop; every
//!   mtime compare fires `MachineTimer`, which re-arms and calls tick_isr().
//!
//! MEMORY BUDGET
//!   Handler + init are tiny; CLINT registers are MMIO (no driver RAM).
//!
//! TICK MATH
//!   QEMU `virt` CLINT timebase = 10 MHz, tick = 1 kHz:
//!   PERIOD = 10_000_000 / 1_000 = 10_000 increments of `mtime`.
//!
//! TRAP SAFETY
//!   mtvec -> _start_trap is installed by `_setup_interrupts`
//!   (see riscv_rt_config.rs). The handler only re-arms the compare and does
//!   one atomic add — never scheduler state (same contract as ARM SysTick).
//! ============================================================================

use crate::arch::{ArchInit, MemoryLayout};

/// Core clock of the CLINT timebase on QEMU `virt` (Hz).
pub const CORE_CLOCK_HZ: u32 = 10_000_000;

// ---------------------------------------------------------------------------
// CLINT tick source
// ---------------------------------------------------------------------------
/// CLINT (Core Local Interruptor) layout on the QEMU `virt` machine.
mod clint {
    pub const BASE: usize = 0x0200_0000;
    /// mtimecmp for hart 0 (64-bit, two 32-bit halves).
    pub const MTIMECMP0: usize = BASE + 0x4000;
    /// Free-running 64-bit counter shared by all harts.
    pub const MTIME: usize = BASE + 0xBFF8;
}

/// Read the 64-bit `mtime` counter safely on RV32 (hi-lo-hi re-read to
/// detect carry between the halves).
#[inline(always)]
fn read_mtime() -> u64 {
    // SAFETY: fixed CLINT MMIO addresses on the QEMU virt machine.
    unsafe {
        loop {
            let hi0 = core::ptr::read_volatile((clint::MTIME + 4) as *const u32) as u64;
            let lo = core::ptr::read_volatile(clint::MTIME as *const u32) as u64;
            let hi1 = core::ptr::read_volatile((clint::MTIME + 4) as *const u32) as u64;
            if hi0 == hi1 {
                return (hi0 << 32) | lo;
            }
        }
    }
}

/// Write `mtimecmp` with the RV32-safe sequence: park the high half at all
/// ones first so the intermediate value can never fire spuriously.
#[inline(always)]
fn write_mtimecmp(value: u64) {
    // SAFETY: fixed CLINT MMIO addresses on the QEMU virt machine.
    unsafe {
        core::ptr::write_volatile((clint::MTIMECMP0 + 4) as *mut u32, 0xFFFF_FFFF);
        core::ptr::write_volatile(clint::MTIMECMP0 as *mut u32, value as u32);
        core::ptr::write_volatile((clint::MTIMECMP0 + 4) as *mut u32, (value >> 32) as u32);
    }
}

/// Schedule the next machine-timer interrupt `PERIOD` ticks from now.
#[inline(always)]
fn arm_next_tick() {
    const PERIOD: u64 = (CORE_CLOCK_HZ as u64) / crate::arch::TICK_RATE_HZ as u64;
    write_mtimecmp(read_mtime() + PERIOD);
}

/// Enable the CLINT machine-timer interrupt at 1 kHz.
pub fn init_tick() {
    arm_next_tick();
    // SAFETY: CSR writes; MTIE/MIE are the documented enable bits.
    unsafe {
        riscv::register::mie::set_mtimer(); // mie.MTIE
        riscv::register::mstatus::set_mie(); // global MIE
    }
}

/// Machine timer interrupt handler — dispatched by riscv-rt from
/// `__INTERRUPTS[7]` after the asm trap prologue saved the frame.
///
/// ISR-safety contract: re-arm the compare, bump the tick latch, return.
/// Never touch scheduler state (see arch::arm SysTick comment).
#[no_mangle]
pub extern "C" fn MachineTimer() {
    arm_next_tick();
    crate::arch::tick_isr();
}

/// RISC-V architecture implementation
pub struct RiscvArch;

impl ArchInit for RiscvArch {
    fn init() {
        // Initialize RISC-V specific features
        Self::irq_init();
        Self::setup_memory_protection();
    }
    
    fn irq_init() {
        // Initialize interrupts for RISC-V
        // For now, just enable basic interrupt handling
    }
    
    fn setup_memory_protection() {
        // Set up PMP if available
        // For now, basic setup
    }
}

/// RISC-V specific memory layout implementation
#[allow(dead_code)]
pub struct RiscvMemoryLayout;

impl MemoryLayout for RiscvMemoryLayout {
    fn ram_start() -> usize {
        0x80000000 // Standard RISC-V RAM start
    }
    
    fn ram_size() -> usize {
        128 * 1024 // 128KB RAM for virt machine
    }
    
    fn flash_start() -> usize {
        0x20000000 // Flash start
    }
    
    fn flash_size() -> usize {
        512 * 1024 // 512KB Flash
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

/// Interrupt control functions for RISC-V
pub fn disable_interrupts() {
    unsafe {
        riscv::register::mstatus::clear_mie();
    }
}

pub fn enable_interrupts() {
    unsafe {
        riscv::register::mstatus::set_mie();
    }
}

/// Early debug output for RISC-V
pub fn early_println(msg: &str) {
    // QEMU virt provides NS16550A UART at 0x1000_0000
    const UART_BASE: usize = 0x1000_0000;
    const THR: usize = UART_BASE + 0; // Transmit holding register
    const LSR: usize = UART_BASE + 5; // Line status register
    const LSR_THRE: u8 = 0x20; // Transmit holding register empty bit
    
    unsafe {
        for byte in msg.bytes() {
            // Wait for UART to be ready to transmit
            while (core::ptr::read_volatile(LSR as *const u8) & LSR_THRE) == 0 {
                // Busy wait - UART not ready
            }
            // Write byte to transmit holding register
            core::ptr::write_volatile(THR as *mut u8, byte);
        }
        // Add newline
        while (core::ptr::read_volatile(LSR as *const u8) & LSR_THRE) == 0 {
            // Busy wait - UART not ready
        }
        core::ptr::write_volatile(THR as *mut u8, b'\n');
    }
}

/// Yield CPU to other tasks (cooperative multitasking)
#[allow(dead_code)]
pub fn yield_cpu() {
    unsafe {
        // RISC-V wait for interrupt instruction
        core::arch::asm!("wfi", options(nomem, nostack));
    }
}

/// Shutdown system
#[allow(dead_code)]
pub fn shutdown() -> ! {
    // Disable interrupts and halt
    unsafe {
        core::arch::asm!("csrci mstatus, 8", options(nomem, nostack));
    }
    
    loop {
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack));
        }
    }
}
