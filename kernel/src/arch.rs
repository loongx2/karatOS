//! Architecture Abstraction Layer
//! Provides unified interface for architecture-specific functionality

#[cfg(target_arch = "arm")]
pub mod arm;
#[cfg(target_arch = "riscv32")]
pub mod riscv;

// Export architecture-specific implementations
#[cfg(target_arch = "arm")]
pub use arm::*;
#[cfg(target_arch = "riscv32")]
pub use riscv::*;

// Fallback implementations for unsupported architectures
#[cfg(not(any(target_arch="arm", target_arch="riscv32")))]
#[inline(always)]
pub fn disable_interrupts() {}

#[cfg(not(any(target_arch="arm", target_arch="riscv32")))]
#[inline(always)]
pub fn enable_interrupts() {}

#[cfg(not(any(target_arch="arm", target_arch="riscv32")))]
#[inline(always)]
pub fn arch_println(_s: &str) {}

#[cfg(not(any(target_arch="arm", target_arch="riscv32")))]
#[inline(always)]
pub fn arch_init() {}

#[cfg(not(any(target_arch="arm", target_arch="riscv32")))]
#[inline(always)]
pub fn arch_yield() {}

#[cfg(not(any(target_arch="arm", target_arch="riscv32")))]
#[inline(always)]
pub fn arch_shutdown() -> ! {
    loop {}
}
