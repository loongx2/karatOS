#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Memory locations to test
const UART_ADDRS: &[usize] = &[
    0x10000000, // QEMU virt UART
    0x10001000, // Alternative
    0x09000000, // Another possibility
    0x101f1000, // SiFive UART
    0x60000000, // Test high memory
];

unsafe fn write_pattern_to_memory(addr: usize, pattern: u32) {
    // Try to write a recognizable pattern
    ptr::write_volatile(addr as *mut u32, pattern);
    ptr::write_volatile((addr + 4) as *mut u32, pattern + 1);
    ptr::write_volatile((addr + 8) as *mut u32, pattern + 2);
    ptr::write_volatile((addr + 12) as *mut u32, pattern + 3);
}

unsafe fn simple_uart_write(addr: usize, data: u8) {
    // Try basic UART write
    ptr::write_volatile(addr as *mut u8, data);
    
    // Try with offset (some UARTs have data register at offset)
    ptr::write_volatile((addr + 1) as *mut u8, data);
    ptr::write_volatile((addr + 4) as *mut u8, data);
    ptr::write_volatile((addr + 8) as *mut u8, data);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        // Write magic patterns to memory first
        for (i, &addr) in UART_ADDRS.iter().enumerate() {
            let pattern = 0xDEADBEE0 + (i as u32);
            write_pattern_to_memory(addr, pattern);
            
            // Try UART output
            simple_uart_write(addr, b'H');
            simple_uart_write(addr, b'E');
            simple_uart_write(addr, b'L');
            simple_uart_write(addr, b'L');
            simple_uart_write(addr, b'O');
            simple_uart_write(addr, b'\n');
        }
        
        // Fill some recognizable memory with patterns
        for i in 0..1000 {
            let addr = 0x80000000 + (i * 4); // Try RAM area
            ptr::write_volatile(addr as *mut u32, 0xBEEFCAFE + i);
        }
        
        // Write to beginning of RAM too
        ptr::write_volatile(0x80000000 as *mut u32, 0x12345678);
        ptr::write_volatile(0x80000004 as *mut u32, 0x9ABCDEF0);
        ptr::write_volatile(0x80000008 as *mut u32, 0xFEDCBA98);
        
        // Infinite loop - keep CPU busy
        loop {
            // Do some work to keep CPU active
            core::arch::asm!("nop");
        }
    }
}
