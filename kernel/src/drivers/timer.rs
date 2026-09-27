//! ============================================================================
//! MODULE : drivers::timer — monotonic time source driver
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   Implements the `Driver` contract for the platform tick timer. The actual
//!   hardware programming (SysTick on ARM, CLINT mtimecmp on RISC-V) lives in
//!   the arch layer because it is core-private, not bus-attached; this driver
//!   exposes the resulting monotonic time through the driver model so kernel
//!   code and (later) modules consume time via the registry, never via arch
//!   internals directly.
//!
//! ROLE IN BOOT FLOW
//!   `PLATFORM_TIMER` singleton in drivers/mod.rs; registered second (after
//!   the console). `arch::init_tick()` must have started the hardware tick
//!   before `read_ticks()` produces non-zero time.
//!
//! TIME MODEL
//!   The arch tick ISR bumps a latch; the main loop drains the latch into
//!   `arch::MILLIS` at 1 kHz (`arch::TICK_RATE_HZ`). ISR-safe by design: the
//!   ISR never touches scheduler state.
//!
//! MEMORY BUDGET
//!   One `u32` (clock hint) per instance + the atomics in arch/mod.rs.
//! ============================================================================

use super::{DeviceClass, Driver, DriverError};

/// Tick timer driver — monotonic milliseconds since boot.
pub struct TickTimerDriver {
    /// Tick input clock (Hz), informational, from the board description.
    clock_hz: u32,
}

impl TickTimerDriver {
    /// Const constructor for the platform singleton.
    pub const fn new() -> Self {
        Self { clock_hz: 0 }
    }

    /// Construct with an explicit clock hint (from a future DTB `clocks=`).
    #[allow(dead_code)]
    pub const fn with_clock(clock_hz: u32) -> Self {
        Self { clock_hz }
    }
}

impl Default for TickTimerDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl Driver for TickTimerDriver {
    fn name(&self) -> &'static str {
        "timer0"
    }

    fn class(&self) -> DeviceClass {
        DeviceClass::Timer
    }

    /// Marks the timer as up. The hardware itself was started by
    /// `arch::init_tick()` just before the registry is initialized.
    fn init(&mut self) -> Result<(), DriverError> {
        if self.clock_hz == 0 {
            // Fill in the arch tick rate as the clock hint on first init.
            self.clock_hz = crate::arch::TICK_RATE_HZ;
        }
        Ok(())
    }

    /// Milliseconds since boot.
    fn read_ticks(&self) -> u64 {
        crate::arch::millis_since_boot() as u64
    }
}