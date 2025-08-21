#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// NS16550A UART register offsets
const UART_BASE: usize = 0x10000000;
const UART_RBR: usize = UART_BASE + 0; // Receiver Buffer Register (read)
const UART_THR: usize = UART_BASE + 0; // Transmitter Holding Register (write)
const UART_DLL: usize = UART_BASE + 0; // Divisor Latch Low (when DLAB=1)
const UART_DLH: usize = UART_BASE + 1; // Divisor Latch High (when DLAB=1)
const UART_IER: usize = UART_BASE + 1; // Interrupt Enable Register
const UART_IIR: usize = UART_BASE + 2; // Interrupt Identification Register (read)
const UART_FCR: usize = UART_BASE + 2; // FIFO Control Register (write)
const UART_LCR: usize = UART_BASE + 3; // Line Control Register
const UART_MCR: usize = UART_BASE + 4; // Modem Control Register
const UART_LSR: usize = UART_BASE + 5; // Line Status Register
const UART_MSR: usize = UART_BASE + 6; // Modem Status Register
const UART_SCR: usize = UART_BASE + 7; // Scratch Register

// Line Control Register bits
const UART_LCR_DLAB: u8 = 0x80; // Divisor Latch Access Bit
const UART_LCR_8N1: u8 = 0x03;  // 8 data bits, no parity, 1 stop bit

// Line Status Register bits
const UART_LSR_THRE: u8 = 0x20; // Transmitter Holding Register Empty

unsafe fn uart_init() {
    // Configure the UART for 115200 baud, 8N1
    // Assuming a 10MHz clock (typical for QEMU)
    
    // Set DLAB to access divisor latches
    ptr::write_volatile(UART_LCR as *mut u8, UART_LCR_DLAB);
    
    // Set divisor for 115200 baud (10MHz / (16 * 115200) = ~5.4, use 5)
    ptr::write_volatile(UART_DLL as *mut u8, 5);
    ptr::write_volatile(UART_DLH as *mut u8, 0);
    
    // Clear DLAB and set 8N1
    ptr::write_volatile(UART_LCR as *mut u8, UART_LCR_8N1);
    
    // Enable FIFOs
    ptr::write_volatile(UART_FCR as *mut u8, 0x01);
    
    // No interrupts
    ptr::write_volatile(UART_IER as *mut u8, 0x00);
    
    // Set RTS and DTR
    ptr::write_volatile(UART_MCR as *mut u8, 0x03);
    
    // Test by sending a character immediately
    ptr::write_volatile(UART_THR as *mut u8, b'!');
}

unsafe fn uart_wait_ready() {
    // Wait until transmitter is ready
    while (ptr::read_volatile(UART_LSR as *const u8) & UART_LSR_THRE) == 0 {
        // Busy wait
    }
}

unsafe fn uart_write_char(c: u8) {
    uart_wait_ready();
    ptr::write_volatile(UART_THR as *mut u8, c);
}

unsafe fn uart_write_string(s: &str) {
    for byte in s.bytes() {
        uart_write_char(byte);
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        // Proper UART initialization
        uart_init();
        
        // Send startup message
        uart_write_string("UART INITIALIZED!\n");
        uart_write_string("RISC-V Kernel Starting...\n");
        uart_write_string("Hello from RISC-V!\n");
        
        // Test continuous output
        let mut counter = 0u32;
        loop {
            uart_write_string("Heartbeat ");
            
            // Simple decimal output
            let mut n = counter;
            let mut digits = [0u8; 10];
            let mut i = 0;
            
            if n == 0 {
                uart_write_char(b'0');
            } else {
                while n > 0 {
                    digits[i] = (n % 10) as u8 + b'0';
                    n /= 10;
                    i += 1;
                }
                while i > 0 {
                    i -= 1;
                    uart_write_char(digits[i]);
                }
            }
            
            uart_write_char(b'\n');
            
            counter += 1;
            
            // Delay
            for _ in 0..1000000 {
                core::arch::asm!("nop");
            }
        }
    }
}
