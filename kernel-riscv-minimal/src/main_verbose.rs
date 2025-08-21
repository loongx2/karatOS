#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// UART configuration for QEMU RISC-V virt machine
const UART_BASE: usize = 0x10000000;
const UART_THR: usize = UART_BASE + 0; // Transmitter Holding Register
const UART_LSR: usize = UART_BASE + 5; // Line Status Register

unsafe fn uart_init() {
    // Basic UART initialization for NS16550A
    // Set up the UART for 8N1 with a baud rate
    
    // First, let's try to read the UART to see if it's there
    let lsr = ptr::read_volatile(UART_LSR as *const u8);
    
    // Write to transmitter holding register directly
    ptr::write_volatile(UART_THR as *mut u8, b'I'); // I for Init
    ptr::write_volatile(UART_THR as *mut u8, b'N');
    ptr::write_volatile(UART_THR as *mut u8, b'I');
    ptr::write_volatile(UART_THR as *mut u8, b'T');
    ptr::write_volatile(UART_THR as *mut u8, b'\n');
}

unsafe fn uart_write_char(c: u8) {
    // Simple write to UART
    ptr::write_volatile(UART_THR as *mut u8, c);
    
    // Small delay
    for _ in 0..1000 {
        core::arch::asm!("nop");
    }
}

unsafe fn uart_write_string(s: &str) {
    for byte in s.bytes() {
        uart_write_char(byte);
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        // Initialize UART
        uart_init();
        
        // Try multiple ways to output
        uart_write_string("HELLO RISC-V!\n");
        uart_write_string("KERNEL STARTED\n");
        uart_write_string("UART TEST\n");
        
        // Also try the other potential UART addresses
        let other_addrs = [0x10001000, 0x09000000, 0x101f1000];
        for &addr in &other_addrs {
            ptr::write_volatile(addr as *mut u8, b'T');
            ptr::write_volatile(addr as *mut u8, b'E');
            ptr::write_volatile(addr as *mut u8, b'S');
            ptr::write_volatile(addr as *mut u8, b'T');
            ptr::write_volatile(addr as *mut u8, b'\n');
        }
        
        // Continuous output
        let mut counter = 0u32;
        loop {
            uart_write_string("ALIVE ");
            counter += 1;
            
            // Write counter (simple hex display)
            let hex_chars = b"0123456789ABCDEF";
            uart_write_char(hex_chars[((counter >> 4) & 0xF) as usize]);
            uart_write_char(hex_chars[(counter & 0xF) as usize]);
            uart_write_char(b'\n');
            
            // Delay
            for _ in 0..100000 {
                core::arch::asm!("nop");
            }
        }
    }
}
