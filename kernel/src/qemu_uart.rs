//! QEMU ARM UART Driver
//! 
//! Simple UART driver for QEMU ARM virtualization (LM3S6965EVB)
//! Provides basic read/write functionality for the virtual UART

use core::ptr::{read_volatile, write_volatile};

/// UART0 base address for LM3S6965EVB in QEMU
const UART0_BASE: usize = 0x4000_C000;

/// UART register offsets
const UART_DR: usize = 0x000;    // Data Register
const UART_FR: usize = 0x018;    // Flag Register
const UART_IBRD: usize = 0x024;  // Integer Baud Rate Divisor
const UART_FBRD: usize = 0x028;  // Fractional Baud Rate Divisor
const UART_LCRH: usize = 0x02C;  // Line Control Register
const UART_CTL: usize = 0x030;   // Control Register

/// UART Flag Register bits
const UART_FR_TXFF: u32 = 1 << 5;  // Transmit FIFO Full
const UART_FR_RXFE: u32 = 1 << 4;  // Receive FIFO Empty

/// UART Control Register bits
const UART_CTL_UARTEN: u32 = 1 << 0;  // UART Enable
const UART_CTL_TXE: u32 = 1 << 8;     // Transmit Enable
const UART_CTL_RXE: u32 = 1 << 9;     // Receive Enable

/// UART Line Control Register bits
const UART_LCRH_WLEN_8: u32 = 0x3 << 5;  // 8-bit word length
const UART_LCRH_FEN: u32 = 1 << 4;       // FIFO Enable

/// QEMU UART Driver
pub struct QemuUart {
    base_addr: usize,
}

impl QemuUart {
    /// Create new UART driver instance
    pub const fn new() -> Self {
        Self {
            base_addr: UART0_BASE,
        }
    }
    
    /// Initialize UART for QEMU
    pub fn init(&self) {
        unsafe {
            // Disable UART
            write_volatile((self.base_addr + UART_CTL) as *mut u32, 0);
            
            // Set baud rate to 115200 (assuming 16MHz clock)
            // Divisor = 16000000 / (16 * 115200) = ~8.68
            write_volatile((self.base_addr + UART_IBRD) as *mut u32, 8);
            write_volatile((self.base_addr + UART_FBRD) as *mut u32, 44);
            
            // Set line control: 8-bit, no parity, 1 stop bit, FIFO enabled
            write_volatile((self.base_addr + UART_LCRH) as *mut u32, 
                         UART_LCRH_WLEN_8 | UART_LCRH_FEN);
            
            // Enable UART, transmit, and receive
            write_volatile((self.base_addr + UART_CTL) as *mut u32, 
                         UART_CTL_UARTEN | UART_CTL_TXE | UART_CTL_RXE);
        }
    }
    
    /// Write a single byte to UART
    pub fn write_byte(&self, byte: u8) {
        unsafe {
            // Wait for transmit FIFO to not be full
            while (read_volatile((self.base_addr + UART_FR) as *const u32) & UART_FR_TXFF) != 0 {
                // Busy wait
            }
            
            // Write byte to data register
            write_volatile((self.base_addr + UART_DR) as *mut u32, byte as u32);
        }
    }
    
    /// Write a string to UART
    pub fn write_str(&self, s: &str) {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r'); // Add carriage return before newline
            }
            self.write_byte(byte);
        }
    }
    
    /// Read a byte from UART (non-blocking)
    pub fn read_byte(&self) -> Option<u8> {
        unsafe {
            // Check if receive FIFO is empty
            if (read_volatile((self.base_addr + UART_FR) as *const u32) & UART_FR_RXFE) != 0 {
                None
            } else {
                // Read byte from data register
                Some((read_volatile((self.base_addr + UART_DR) as *const u32) & 0xFF) as u8)
            }
        }
    }
    
    /// Check if data is available to read
    pub fn data_available(&self) -> bool {
        unsafe {
            (read_volatile((self.base_addr + UART_FR) as *const u32) & UART_FR_RXFE) == 0
        }
    }
    
    /// Check if transmit is ready
    #[allow(dead_code)]
    pub fn tx_ready(&self) -> bool {
        unsafe {
            (read_volatile((self.base_addr + UART_FR) as *const u32) & UART_FR_TXFF) == 0
        }
    }
}

/// Write implementation for formatting support
impl core::fmt::Write for QemuUart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        QemuUart::write_str(self, s);
        Ok(())
    }
}

/// Global UART instance
static mut QEMU_UART: QemuUart = QemuUart::new();

/// Initialize global UART
#[allow(static_mut_refs)]
pub fn init_uart() {
    unsafe {
        let uart = &mut QEMU_UART;
        uart.init();
    }
}

/// Write string to global UART
#[allow(static_mut_refs)]
pub fn uart_write_str(s: &str) {
    unsafe {
        let uart = &QEMU_UART;
        uart.write_str(s);
    }
}

/// Write byte to global UART
#[allow(dead_code, static_mut_refs)]
pub fn uart_write_byte(byte: u8) {
    unsafe {
        let uart = &QEMU_UART;
        uart.write_byte(byte);
    }
}

/// Read byte from global UART
#[allow(static_mut_refs)]
pub fn uart_read_byte() -> Option<u8> {
    unsafe {
        let uart = &QEMU_UART;
        uart.read_byte()
    }
}

/// Check if UART data is available
#[allow(static_mut_refs)]
pub fn uart_data_available() -> bool {
    unsafe {
        let uart = &QEMU_UART;
        uart.data_available()
    }
}

/// UART print macro
#[macro_export]
macro_rules! uart_print {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut uart = unsafe { &mut crate::qemu_uart::QEMU_UART };
            let _ = write!(uart, $($arg)*);
        }
    };
}

/// UART println macro
#[macro_export]
macro_rules! uart_println {
    () => (uart_print!("\n"));
    ($($arg:tt)*) => (uart_print!("{}\n", format_args!($($arg)*)));
}
