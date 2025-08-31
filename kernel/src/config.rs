//! Build configuration for multi-architecture kernel
//! Provides build-time configuration for different targets

/// Target architecture information
pub struct TargetInfo {
    pub arch: &'static str,
    pub vendor: &'static str,
    pub os: &'static str,
    pub env: &'static str,
}

/// Get target information for the current build
pub const fn get_target_info() -> TargetInfo {
    #[cfg(all(target_arch = "arm", target_vendor = "unknown", target_os = "none"))]
    {
        TargetInfo {
            arch: "arm",
            vendor: "unknown",
            os: "none",
            env: "eabi",
        }
    }
    
    #[cfg(all(target_arch = "riscv32", target_vendor = "unknown", target_os = "none"))]
    {
        TargetInfo {
            arch: "riscv32",
            vendor: "unknown", 
            os: "none",
            env: "elf",
        }
    }

    // Default fallback for host targets (testing, etc.)
    #[cfg(not(any(
        all(target_arch = "arm", target_vendor = "unknown", target_os = "none"),
        all(target_arch = "riscv32", target_vendor = "unknown", target_os = "none")
    )))]
    {
        TargetInfo {
            arch: "unknown",
            vendor: "unknown",
            os: "unknown", 
            env: "unknown",
        }
    }
}

/// Build-time feature configuration
pub struct BuildConfig {
    pub has_fpu: bool,
    pub has_mmu: bool,
    pub pointer_width: usize,
    pub endianness: &'static str,
}

/// Get build configuration for the current target
pub const fn get_build_config() -> BuildConfig {
    BuildConfig {
        // Assume no FPU for embedded targets
        has_fpu: false,
        
        // Embedded targets typically don't have MMU
        has_mmu: false,
        
        pointer_width: core::mem::size_of::<usize>() * 8,
        
        #[cfg(target_endian = "little")]
        endianness: "little",
        #[cfg(target_endian = "big")]
        endianness: "big",
    }
}
