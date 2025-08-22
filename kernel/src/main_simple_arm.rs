#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// ARM Cortex-M Vector Table using a union to work around const limitations
#[repr(C)]
union VectorEntry {
    handler: unsafe extern "C" fn() -> !,
    stack_ptr: u32,
}

#[link_section = ".vector_table"]
#[no_mangle]
pub static VECTOR_TABLE: [VectorEntry; 16] = [
    VectorEntry { stack_ptr: 0x20010000 },  // 0: Initial Stack Pointer
    VectorEntry { handler: reset_handler }, // 1: Reset Handler  
    VectorEntry { handler: default_handler }, // 2: NMI
    VectorEntry { handler: default_handler }, // 3: Hard Fault
    VectorEntry { handler: default_handler }, // 4: Memory Management Fault
    VectorEntry { handler: default_handler }, // 5: Bus Fault
    VectorEntry { handler: default_handler }, // 6: Usage Fault
    VectorEntry { stack_ptr: 0 },           // 7: Reserved
    VectorEntry { stack_ptr: 0 },           // 8: Reserved
    VectorEntry { stack_ptr: 0 },           // 9: Reserved
    VectorEntry { stack_ptr: 0 },           // 10: Reserved
    VectorEntry { handler: default_handler }, // 11: SVC
    VectorEntry { handler: default_handler }, // 12: Debug Monitor
    VectorEntry { stack_ptr: 0 },           // 13: Reserved
    VectorEntry { handler: default_handler }, // 14: PendSV
    VectorEntry { handler: default_handler }, // 15: SysTick
];

unsafe extern "C" fn default_handler() -> ! {
    loop {}
}

#[no_mangle]
unsafe extern "C" fn reset_handler() -> ! {
    // Set up stack pointer to top of RAM
    core::arch::asm!(
        "ldr sp, =0x20010000",
        "bl {main}",
        main = sym main,
        options(noreturn)
    );
}

#[no_mangle]
extern "C" fn main() -> ! {
    // LM3S6965EVB UART0 registers
    const UART0_BASE: usize = 0x4000C000;
    const UARTDR: usize = UART0_BASE + 0x000;  // Data Register
    const UARTFR: usize = UART0_BASE + 0x018;  // Flag Register
    
    // Send messages matching RISC-V format
    let message1 = b"ARM kernel started!\r\n";
    let message2 = b"Architecture: ARM Cortex-M3\r\n";
    let message3 = b"Board: LM3S6965EVB\r\n";
    let message4 = b"karatOS ARM platform working!\r\n";
    
    unsafe {
        for &byte in message1 {
            while (*(UARTFR as *const u32) & 0x20) != 0 {}
            *(UARTDR as *mut u32) = byte as u32;
        }
        for &byte in message2 {
            while (*(UARTFR as *const u32) & 0x20) != 0 {}
            *(UARTDR as *mut u32) = byte as u32;
        }
        for &byte in message3 {
            while (*(UARTFR as *const u32) & 0x20) != 0 {}
            *(UARTDR as *mut u32) = byte as u32;
        }
        for &byte in message4 {
            while (*(UARTFR as *const u32) & 0x20) != 0 {}
            *(UARTDR as *mut u32) = byte as u32;
        }
    }
    
    loop {}
}
