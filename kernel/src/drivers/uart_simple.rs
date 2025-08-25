//! Universal UART driver
//! Architecture-agnostic UART interface for kernel output

/// Print a string to the debug console
pub fn print(s: &str) {
    for byte in s.bytes() {
        print_char(byte);
    }
}

/// Print a single character to the debug console
#[cfg(target_arch = "arm")]
fn print_char(c: u8) {
    // ARM: Use semihosting for output
    use cortex_m_semihosting::hprint;
    let _ = hprint!("{}", c as char);
}

#[cfg(target_arch = "riscv32")]
fn print_char(c: u8) {
    // RISC-V: Use memory-mapped UART
    const UART_BASE: *mut u8 = 0x10000000 as *mut u8;
    
    unsafe {
        // Wait for transmit holding register to be empty
        while ((UART_BASE.add(5) as *mut u8).read_volatile() & 0x20) == 0 {}
        // Write byte to transmit holding register
        (UART_BASE as *mut u8).write_volatile(c);
    }
}
