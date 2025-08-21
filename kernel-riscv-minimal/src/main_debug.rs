#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Write panic signature to memory
    unsafe {
        ptr::write_volatile(0x80001000 as *mut u32, 0xDEADDEAD);
        ptr::write_volatile(0x80001004 as *mut u32, 0xBEEFBEEF);
    }
    loop {}
}

const UART_BASE: usize = 0x10000000;
const UART_THR: usize = UART_BASE + 0; // Transmitter Holding Register
const UART_LSR: usize = UART_BASE + 5; // Line Status Register
const UART_LCR: usize = UART_BASE + 3; // Line Control Register
const UART_DLL: usize = UART_BASE + 0; // Divisor Latch Low (when DLAB=1)
const UART_DLH: usize = UART_BASE + 1; // Divisor Latch High (when DLAB=1)
const UART_FCR: usize = UART_BASE + 2; // FIFO Control Register
const UART_IER: usize = UART_BASE + 1; // Interrupt Enable Register
const UART_MCR: usize = UART_BASE + 4; // Modem Control Register

#[no_mangle]
#[link_section = ".text._start"]
pub extern "C" fn _start() -> ! {
    unsafe {
        // Write startup signature to memory that we can verify with QEMU monitor
        ptr::write_volatile(0x80000100 as *mut u32, 0x12345678); // Startup marker
        ptr::write_volatile(0x80000104 as *mut u32, 0x87654321); // Second marker
        
        // Try the simplest possible UART setup
        // Just write directly to UART THR without initialization
        ptr::write_volatile(UART_THR as *mut u8, b'A');
        ptr::write_volatile(UART_THR as *mut u8, b'B');
        ptr::write_volatile(UART_THR as *mut u8, b'C');
        ptr::write_volatile(UART_THR as *mut u8, b'\n');
        
        // Now try with basic initialization
        // Set 8N1 mode
        ptr::write_volatile(UART_LCR as *mut u8, 0x03);
        
        // Send more characters
        ptr::write_volatile(UART_THR as *mut u8, b'H');
        ptr::write_volatile(UART_THR as *mut u8, b'E');
        ptr::write_volatile(UART_THR as *mut u8, b'L');
        ptr::write_volatile(UART_THR as *mut u8, b'L');
        ptr::write_volatile(UART_THR as *mut u8, b'O');
        ptr::write_volatile(UART_THR as *mut u8, b'\n');
        
        // Update progress markers
        ptr::write_volatile(0x80000108 as *mut u32, 0xCAFEBABE); // Progress marker
        
        // Try a more complete UART init
        // Access divisor latches
        ptr::write_volatile(UART_LCR as *mut u8, 0x80); // Set DLAB
        ptr::write_volatile(UART_DLL as *mut u8, 1);    // Low divisor
        ptr::write_volatile(UART_DLH as *mut u8, 0);    // High divisor
        ptr::write_volatile(UART_LCR as *mut u8, 0x03); // Clear DLAB, set 8N1
        
        // Enable FIFO
        ptr::write_volatile(UART_FCR as *mut u8, 0x01);
        
        // Send test message
        let msg = b"UART_READY\n";
        for &byte in msg {
            ptr::write_volatile(UART_THR as *mut u8, byte);
        }
        
        // Final progress marker
        ptr::write_volatile(0x8000010C as *mut u32, 0xDEADBEEF);
        
        // Keep trying to send data
        let mut counter = 0u8;
        loop {
            ptr::write_volatile(UART_THR as *mut u8, b'0' + (counter % 10));
            counter = counter.wrapping_add(1);
            
            // Small delay
            for _ in 0..100000 {
                core::arch::asm!("nop");
            }
            
            // Update counter in memory
            ptr::write_volatile(0x80000110 as *mut u32, counter as u32);
        }
    }
}
