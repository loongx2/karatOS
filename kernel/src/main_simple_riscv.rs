#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// RISC-V entry point - placed at start of .text section
#[no_mangle]
#[link_section = ".text._start"]
pub unsafe extern "C" fn _start() -> ! {
    // Set up stack pointer to top of RAM and jump to main
    core::arch::asm!(
        "li sp, 0x88000000",  // Stack at top of 128MB RAM
        "call {main}",        // Call main function
        main = sym main,
        options(noreturn)
    );
}

#[no_mangle]
fn main() -> ! {
    // QEMU RISC-V 'virt' machine UART0 base address
    const UART_BASE: *mut u8 = 0x10000000 as *mut u8;
    
    let message = b"RISC-V kernel started!\n\r";
    
    unsafe {
        for &byte in message {
            // Wait for transmit holding register to be empty
            while ((UART_BASE.add(5) as *mut u8).read_volatile() & 0x20) == 0 {}
            // Write byte to transmit holding register
            (UART_BASE as *mut u8).write_volatile(byte);
        }
    }
    
    loop {}
}
