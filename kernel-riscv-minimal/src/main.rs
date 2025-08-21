#![no_std]
#![no_main]

use panic_halt as _;

// QEMU exit mechanism (for testing)
// RISC-V virt machine uses SiFive test device for exit
const QEMU_EXIT_SUCCESS: u32 = 0x5555;
const QEMU_EXIT_FAILURE: u32 = 0x3333;
const SIFIVE_TEST_BASE: usize = 0x100000;

unsafe fn qemu_exit(exit_code: u32) {
    // Use SiFive test device for clean QEMU exit
    let test_device = SIFIVE_TEST_BASE as *mut u32;
    test_device.write_volatile(exit_code);
}

// UART base address for QEMU virt machine (ns16550a)
const UART_BASE: usize = 0x10000000;

// UART registers (16550A compatible)
const UART_THR: usize = 0; // Transmit Holding Register
const UART_RBR: usize = 0; // Receive Buffer Register
const UART_IER: usize = 1; // Interrupt Enable Register
const UART_FCR: usize = 2; // FIFO Control Register
const UART_LCR: usize = 3; // Line Control Register
const UART_MCR: usize = 4; // Modem Control Register
const UART_LSR: usize = 5; // Line Status Register
const UART_MSR: usize = 6; // Modem Status Register

// Initialize UART for basic operation
unsafe fn uart_init() {
    let uart_base = UART_BASE as *mut u8;
    
    // Set line control: 8 bits, no parity, 1 stop bit
    uart_base.add(UART_LCR).write_volatile(0x03);
    
    // Enable FIFO, clear them, with 14-byte threshold
    uart_base.add(UART_FCR).write_volatile(0xC7);
    
    // Enable auxiliary output 2 (used as interrupt line for CPU)
    uart_base.add(UART_MCR).write_volatile(0x0B);
    
    // Disable all interrupts
    uart_base.add(UART_IER).write_volatile(0x00);
}

// Simple UART functions
unsafe fn uart_write_char(c: u8) {
    let uart_base = UART_BASE as *mut u8;
    
    // Wait for transmitter to be ready (with timeout)
    for _ in 0..100000 {
        let lsr = uart_base.add(UART_LSR).read_volatile();
        if (lsr & 0x20) != 0 { // THRE bit
            break;
        }
    }
    
    // Write character
    uart_base.add(UART_THR).write_volatile(c);
}

unsafe fn uart_write_str(s: &str) {
    for byte in s.bytes() {
        uart_write_char(byte);
        if byte == b'\n' {
            uart_write_char(b'\r'); // Add carriage return for better display
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    // Initialize UART first
    uart_init();
    
    // Send a simple test character to verify UART is working
    uart_write_char(b'H');
    uart_write_char(b'i');
    uart_write_char(b'!');
    uart_write_char(b'\n');
    
    // Test that we can execute code by doing some computation
    let mut test_value = 42u32;
    test_value = test_value.wrapping_mul(2);
    test_value = test_value.wrapping_add(1);
    // test_value should now be 85
    
    // Try UART output
    uart_write_str("*** RISC-V RTOS Kernel Starting ***\n");
    uart_write_str("Architecture: RISC-V 32-bit\n");
    uart_write_str("Platform: QEMU virt machine\n");
    uart_write_str("Test computation result: ");
    
    // Simple number display (expecting 85)
    if test_value >= 80 {
        uart_write_char(b'8');
        test_value -= 80;
    }
    uart_write_char(test_value as u8 + b'0');
    uart_write_str("\n");
    
    uart_write_str("Kernel test completed successfully!\n");
    uart_write_str("Attempting to exit QEMU...\n");
    
    // Try multiple exit methods
    // Method 1: SiFive test device
    let test_device = SIFIVE_TEST_BASE as *mut u32;
    test_device.write_volatile(QEMU_EXIT_SUCCESS);
    
    // Method 2: If that didn't work, try the alternative address
    let alt_exit = 0x101000 as *mut u32;
    alt_exit.write_volatile(QEMU_EXIT_SUCCESS);
    
    uart_write_str("Exit commands sent, entering infinite loop...\n");
    
    // Should never reach here, but just in case
    loop {
        core::hint::spin_loop();
    }
}
