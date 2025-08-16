#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};

#[entry]
fn main() -> ! {
    hprintln!("Hello from ARM minimal RTOS!");
    
    // Simple test loop with debug output
    for i in 0..3 {
        hprintln!("Loop iteration: {}", i);
    }
    
    hprintln!("Test completed successfully!");
    
    // Exit QEMU
    debug::exit(debug::EXIT_SUCCESS);
    
    loop {}
}
