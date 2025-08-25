//! Memory layout configuration
//! Architecture-agnostic memory layout definitions

/// Get memory regions for the current target
pub fn get_memory_regions() -> MemoryRegions {
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    {
        MemoryRegions {
            ram_start: 0x20000000,
            ram_size: 64 * 1024,
            flash_start: 0x00000000,
            flash_size: 256 * 1024,
        }
    }
    
    #[cfg(all(target_arch = "riscv32", target_os = "none"))]
    {
        MemoryRegions {
            ram_start: 0x80000000,
            ram_size: 128 * 1024,
            flash_start: 0x20000000,
            flash_size: 512 * 1024,
        }
    }
    
    #[cfg(not(any(
        all(target_arch = "arm", target_os = "none"),
        all(target_arch = "riscv32", target_os = "none")
    )))]
    {
        MemoryRegions {
            ram_start: 0,
            ram_size: 0,
            flash_start: 0,
            flash_size: 0,
        }
    }
}

/// Common memory regions used by the kernel
pub struct MemoryRegions {
    pub ram_start: usize,
    pub ram_size: usize,
    pub flash_start: usize,
    pub flash_size: usize,
}

impl MemoryRegions {
    pub fn ram_end(&self) -> usize {
        self.ram_start + self.ram_size
    }
    
    pub fn flash_end(&self) -> usize {
        self.flash_start + self.flash_size
    }
    
    pub fn stack_top(&self) -> usize {
        self.ram_start + self.ram_size
    }
    
    pub fn heap_start(&self) -> usize {
        self.ram_start + (self.ram_size / 2)
    }
    
    pub fn heap_size(&self) -> usize {
        self.ram_size / 4
    }
}
