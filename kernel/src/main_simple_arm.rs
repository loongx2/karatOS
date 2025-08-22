#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};

#[entry]
fn main() -> ! {
    hprintln!("ARM Cortex-M kernel started!");
    hprintln!("Architecture: ARM Cortex-M3");
    hprintln!("Board: LM3S6965EVB");
    hprintln!("karatOS ARM platform working!");

    let mut counter = 0u32;
    loop {
        counter += 1;
        if counter % 1000000 == 0 {
            hprintln!("ARM heartbeat: {}", counter / 1000000);
        }
        
        // Stop after a few heartbeats for testing
        if counter >= 10000000 {
            hprintln!("ARM kernel test completed successfully!");
            debug::exit(debug::EXIT_SUCCESS);
        }
    }
}
