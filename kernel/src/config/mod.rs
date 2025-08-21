//! Platform Configuration Module
//! Provides unified interface for platform-specific configurations

pub mod arm;
pub mod riscv;

use crate::drivers::DeviceConfig;

/// Platform abstraction trait
pub trait PlatformConfig {
    fn device_config() -> DeviceConfig;
    fn platform_init();
}

// Export the correct platform configuration based on target
#[cfg(target_arch = "arm")]
pub use arm::ArmConfig as PlatformImpl;
#[cfg(target_arch = "riscv32")]
pub use riscv::RiscvConfig as PlatformImpl;

// Implement PlatformConfig for the active platform
#[cfg(target_arch = "arm")]
impl PlatformConfig for arm::ArmConfig {
    fn device_config() -> DeviceConfig {
        arm::ArmConfig::device_config()
    }
    
    fn platform_init() {
        arm::ArmConfig::platform_init()
    }
}

#[cfg(target_arch = "riscv32")]
impl PlatformConfig for riscv::RiscvConfig {
    fn device_config() -> DeviceConfig {
        riscv::RiscvConfig::device_config()
    }
    
    fn platform_init() {
        riscv::RiscvConfig::platform_init()
    }
}

/// Get the current platform's device configuration
pub fn get_device_config() -> DeviceConfig {
    PlatformImpl::device_config()
}

/// Initialize the current platform
pub fn init_platform() {
    PlatformImpl::platform_init()
}
