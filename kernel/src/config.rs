//! Configuration management for the karatOS kernel

/// Target platform information
#[allow(dead_code)]
pub struct TargetInfo {
    pub arch: &'static str,
    pub platform: &'static str,
    pub features: &'static [&'static str],
}

/// Get target platform information
#[allow(dead_code)]
pub const fn get_target_info() -> TargetInfo {
    #[cfg(feature = "arm")]
    {
        TargetInfo {
            arch: "ARM Cortex-M",
            platform: "thumbv7m-none-eabi",
            features: &["arm", "cortex-m"],
        }
    }
    
    #[cfg(feature = "riscv")]
    {
        TargetInfo {
            arch: "RISC-V",
            platform: "riscv32imac-unknown-none-elf",
            features: &["riscv", "riscv32"],
        }
    }
    
    #[cfg(not(any(feature = "arm", feature = "riscv")))]
    {
        TargetInfo {
            arch: "Host",
            platform: "host",
            features: &["std"],
        }
    }
}

/// Runtime configuration for debugging and monitoring
#[allow(dead_code)]
pub struct RuntimeConfig {
    pub enable_scheduler_stats: bool,
    pub enable_debug_output: bool,
    pub max_tasks: usize,
    pub timer_frequency: u32,
}

/// Get runtime configuration
#[allow(dead_code)]
pub const fn get_runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        enable_scheduler_stats: true,
        enable_debug_output: true,
        max_tasks: 8,
        timer_frequency: 1000, // 1KHz
    }
}

/// Build configuration options
#[allow(dead_code)]
pub struct BuildConfig {
    pub has_fpu: bool,
    pub has_mmu: bool,
    pub pointer_width: usize,
    pub endianness: &'static str,
}

/// Get build configuration for the current target
#[allow(dead_code)]
pub const fn get_build_config() -> BuildConfig {
    BuildConfig {
        has_fpu: false, // Embedded targets typically don't have FPU enabled
        has_mmu: false, // Neither ARM Cortex-M nor our RISC-V target have MMU
        pointer_width: core::mem::size_of::<usize>() * 8,
        
        #[cfg(target_endian = "little")]
        endianness: "little",
        
        #[cfg(target_endian = "big")]
        endianness: "big",
    }
}
