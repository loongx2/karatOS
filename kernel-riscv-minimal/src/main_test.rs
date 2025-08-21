#![no_std]
#![no_main]

use panic_halt as _;

// Simple test that modifies memory in a pattern we can detect
const TEST_MEMORY: usize = 0x80100000;

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    // Write a recognizable pattern to memory to prove we're running
    let test_ptr = TEST_MEMORY as *mut u32;
    
    // Write magic values that we can check for
    test_ptr.write_volatile(0xDEADBEEF);
    test_ptr.offset(1).write_volatile(0xCAFEBABE);
    test_ptr.offset(2).write_volatile(0x12345678);
    test_ptr.offset(3).write_volatile(0x87654321);
    
    // Try multiple different UART addresses - maybe we have the wrong one
    let uart_addresses = [
        0x10000000, // Standard QEMU virt
        0x10001000, // Alternative
        0x09000000, // Another possibility
        0x101f1000, // Yet another
    ];
    
    for &uart_addr in &uart_addresses {
        let uart_ptr = uart_addr as *mut u8;
        
        // Try to write "HELLO" to each possible UART
        uart_ptr.write_volatile(b'H');
        uart_ptr.write_volatile(b'E');
        uart_ptr.write_volatile(b'L');
        uart_ptr.write_volatile(b'L');
        uart_ptr.write_volatile(b'O');
        uart_ptr.write_volatile(b'\n');
        
        // Wait a bit
        for _ in 0..100000 {
            core::hint::spin_loop();
        }
    }
    
    // Infinite loop with memory updates to show we're alive
    let mut counter = 0u32;
    loop {
        counter = counter.wrapping_add(1);
        test_ptr.offset(4).write_volatile(counter);
        
        // Wait
        for _ in 0..1000000 {
            core::hint::spin_loop();
        }
        
        // Also try writing to UART periodically
        let uart_ptr = 0x10000000 as *mut u8;
        uart_ptr.write_volatile(b'.');
    }
}
