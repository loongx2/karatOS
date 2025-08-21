#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

const UART_BASE: usize = 0x10000000;

#[no_mangle]
#[link_section = ".text._start"]
pub extern "C" fn _start() -> ! {
    unsafe {
        // Just blast characters to UART as fast as possible
        // No initialization, no waiting, just raw writes
        loop {
            ptr::write_volatile(UART_BASE as *mut u8, b'A');
            ptr::write_volatile(UART_BASE as *mut u8, b'B');
            ptr::write_volatile(UART_BASE as *mut u8, b'C');
            ptr::write_volatile(UART_BASE as *mut u8, b'\n');
            
            // Very short delay
            for _ in 0..10000 {
                core::arch::asm!("nop");
            }
        }
    }
}
