// ARM Cortex-M minimal context switch stub

#[cfg(feature = "arm")]
use cortex_m;

#[no_mangle]
pub extern "C" fn arch_init() {
    // Setup stack, interrupts, etc.
    #[cfg(feature = "arm")]
    {
        // Basic ARM Cortex-M initialization
        // Enable NVIC if needed
    }
}

#[no_mangle]
pub extern "C" fn context_switch() {
    // Save/restore context (stub)
}

#[inline(always)]
pub fn disable_interrupts() {
    #[cfg(feature = "arm")]
    cortex_m::interrupt::disable();
}

#[inline(always)]
pub fn enable_interrupts() {
    #[cfg(feature = "arm")]
    unsafe { cortex_m::interrupt::enable(); }
}

#[inline(always)]
pub fn arch_println(s: &str) {
    // Basic semihosting output if feature enabled
    #[cfg(feature = "arm")]
    {
        use cortex_m_semihosting::hio;
        if let Ok(mut h) = hio::hstdout() { use core::fmt::Write; let _ = h.write_str(s); let _ = h.write_str("\n"); }
    }
    let _ = s; // suppress unused warning without feature
}
