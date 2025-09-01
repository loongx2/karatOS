//! Embassy Time Driver for karatOS
//!
//! This module provides a basic time driver for Embassy executor
//! using the system's existing timer infrastructure.

use core::sync::atomic::{AtomicU32, Ordering};

// Simple time source using system ticks
static TICKS: AtomicU32 = AtomicU32::new(0);

/// Initialize Embassy time driver
pub fn init() {
    // Embassy time driver is initialized automatically
}

/// Get current time in ticks
pub fn now() -> u32 {
    TICKS.fetch_add(1, Ordering::Relaxed)
}

/// Convert Duration to system ticks
pub fn duration_to_ticks(duration: embassy_time::Duration) -> u64 {
    // Simple conversion - should be calibrated for actual timer frequency
    duration.as_millis() as u64 * 1000
}

/// Convert system ticks to Duration
pub fn ticks_to_duration(ticks: u64) -> embassy_time::Duration {
    embassy_time::Duration::from_micros(ticks / 1000)
}
