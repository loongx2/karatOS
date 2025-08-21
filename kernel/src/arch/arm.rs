//! ARM Architecture Implementation
//! ARM-specific functionality and hardware abstraction

#[cfg(feature = "arm")]
use cortex_m;
#[cfg(feature = "arm")]
use core::fmt::Write;

#[no_mangle]
pub extern "C" fn arch_init() {
    // ARM-specific initialization
    #[cfg(feature = "arm")]
    {
        // Initialize ARM Cortex-M features
        // NVIC, SCB, etc. configuration goes here
    }
}

#[no_mangle]
pub extern "C" fn context_switch() {
    // ARM context switch implementation
    // Save/restore CPU state
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

pub fn arch_println(s: &str) {
    // ARM-specific debug output
    // This would typically use semihosting or a debug UART
    #[cfg(feature = "arm")]
    {
        // Use semihosting for debug output
        use cortex_m_semihosting::hio;
        if let Ok(mut stdout) = hio::hstdout() {
            let _ = writeln!(stdout, "{}", s);
        }
    }
    #[cfg(not(feature = "arm"))]
    {
        let _ = s; // Suppress unused warning
    }
}

#[inline(always)]
pub fn arch_yield() {
    #[cfg(feature = "arm")]
    cortex_m::asm::wfi(); // Wait for interrupt
}

#[inline(always)]
pub fn arch_shutdown() -> ! {
    #[cfg(feature = "arm")]
    {
        use cortex_m_semihosting::debug;
        debug::exit(debug::EXIT_SUCCESS);
        loop {} // This line is unreachable but needed for type safety
    }
    #[cfg(not(feature = "arm"))]
    loop {}
}
