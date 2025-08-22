#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// ARM Cortex-M entry point - simple approach like RISC-V
#[no_mangle]
#[link_section = ".text._start"]
pub unsafe extern "C" fn _start() -> ! {
    // Set up stack pointer to top of RAM (LM3S6965EVB has 64KB RAM at 0x20000000)
    core::arch::asm!(
        "ldr sp, =0x20010000",  // Stack at top of 64KB RAM
        "bl {main}",            // Call main function
        main = sym main,
        options(noreturn)
    );
}

#[no_mangle]
fn main() -> ! {
    // Simple test using semihosting first
    use core::fmt::Write;
    
    // Create a simple writer that just loops
    struct DummyWriter;
    impl Write for DummyWriter {
        fn write_str(&mut self, _s: &str) -> core::fmt::Result {
            Ok(())
        }
    }
    
    let mut writer = DummyWriter;
    let _ = writeln!(writer, "ARM kernel started!");
    
    // Now try direct message without UART complications
    loop {}
}
