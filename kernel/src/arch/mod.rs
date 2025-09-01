//! Architecture abstraction layer
//! Provides common interface for ARM and RISC-V architectures

#[cfg(target_arch = "arm")]
pub mod arm;
#[cfg(target_arch = "riscv32")]
pub mod riscv;

/// Common architecture initialization trait
pub trait ArchInit {
    fn init();
    fn irq_init();
    fn setup_memory_protection();
}

/// Memory layout trait for architecture-specific memory maps
pub trait MemoryLayout {
    fn ram_start() -> usize;
    fn ram_size() -> usize;
    fn flash_start() -> usize;  
    fn flash_size() -> usize;
    fn stack_top() -> usize;
    fn heap_start() -> usize;
    fn heap_size() -> usize;
}

/// Architecture abstraction - unified interface
pub struct Architecture;

impl Architecture {
    pub fn init() {
        #[cfg(target_arch = "arm")]
        arm::ArmArch::init();
        
        #[cfg(target_arch = "riscv32")]
        riscv::RiscvArch::init();
    }
}

/// Early debug output (available before full driver initialization)
pub fn early_println(_msg: &str) {
    #[cfg(target_arch = "arm")]
    arm::early_println(_msg);
    
    #[cfg(target_arch = "riscv32")]
    riscv::early_println(_msg);
}

/// Architecture-specific yield/wait instruction
pub fn arch_yield() {
    #[cfg(target_arch = "arm")]
    unsafe { core::arch::asm!("wfi") };
    
    #[cfg(target_arch = "riscv32")]
    unsafe { core::arch::asm!("wfi") };
}

/// Architecture-specific wait for interrupt (power-saving)
pub fn wait_for_interrupt() {
    #[cfg(target_arch = "arm")]
    unsafe { core::arch::asm!("wfe") }; // ARM uses WFE (Wait For Event)
    
    #[cfg(target_arch = "riscv32")]
    unsafe { core::arch::asm!("wfi") }; // RISC-V uses WFI (Wait For Interrupt)
}

/// Interrupt control functions
pub fn disable_interrupts() {
    #[cfg(target_arch = "arm")]
    arm::disable_interrupts();
    
    #[cfg(target_arch = "riscv32")]
    riscv::disable_interrupts();
}

pub fn enable_interrupts() {
    #[cfg(target_arch = "arm")]
    arm::enable_interrupts();
    
    #[cfg(target_arch = "riscv32")]
    riscv::enable_interrupts();
}

/// Architecture shutdown
pub fn arch_shutdown() -> ! {
    loop {
        arch_yield();
    }
}

/// Panic handler for RISC-V only (ARM uses panic-halt)
#[cfg(target_arch = "riscv32")]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    // Try to print panic info
    if let Some(location) = info.location() {
        early_println("PANIC at: ");
        early_println(location.file());
    } else {
        early_println("PANIC occurred");
    }
    
    arch_shutdown()
}
