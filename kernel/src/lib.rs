#![no_std]
#![no_main]

pub mod arch;
pub mod scheduler;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
