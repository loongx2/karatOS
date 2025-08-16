// RISC-V minimal context switch stub
#[no_mangle]
pub extern "C" fn arch_init() {
    // Setup stack, interrupts, etc.
}

#[no_mangle]
pub extern "C" fn context_switch() {
    // Save/restore context (stub)
}

#[inline(always)]
pub fn disable_interrupts() {
    // RISC-V: clear MIE bit in mstatus via inline asm (stub)
}

#[inline(always)]
pub fn enable_interrupts() {
    // RISC-V: set MIE bit in mstatus via inline asm (stub)
}

const UART0: *mut u8 = 0x1000_0000 as *mut u8; // QEMU virt UART0 (ns16550)

#[inline(always)]
unsafe fn uart_put(b: u8) {
    // very naive: no LSR polling
    core::ptr::write_volatile(UART0, b);
}

pub fn arch_println(s: &str) {
    for &b in s.as_bytes() { unsafe { uart_put(b); } }
    unsafe { uart_put(b'\n'); }
}
