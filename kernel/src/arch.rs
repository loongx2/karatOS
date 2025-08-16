#[cfg(target_arch = "arm")]
pub mod arm;
#[cfg(target_arch = "riscv32")]
pub mod riscv;

#[cfg(target_arch = "arm")]
pub use arm::{arch_init, arch_println};
#[cfg(target_arch = "riscv32")]
pub use riscv::{arch_init, arch_println};

// Export interrupt functions only when needed
#[cfg(target_arch = "arm")]
pub use arm::{disable_interrupts, enable_interrupts};
#[cfg(target_arch = "riscv32")]
pub use riscv::{disable_interrupts, enable_interrupts};

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
