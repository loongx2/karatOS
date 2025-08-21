//! UART Driver Module
//! Unified UART driver that supports multiple UART types based on device configuration

use super::{Driver, DeviceConfig};
use core::ptr;

/// Unified UART driver that adapts to different hardware
pub struct UartDriver {
    base_addr: usize,
    uart_type: UartType,
}

#[derive(Debug, Clone, Copy)]
enum UartType {
    Pl011,      // ARM PL011 UART
    Ns16550a,   // NS16550A compatible UART (RISC-V)
}

#[derive(Debug)]
pub enum UartError {
    UnsupportedType,
    InitializationFailed,
}

impl Driver for UartDriver {
    type Error = UartError;
    
    fn init(config: &DeviceConfig) -> Result<Self, Self::Error> {
        let uart_type = match config.uart_type {
            "pl011" => UartType::Pl011,
            "ns16550a" => UartType::Ns16550a,
            _ => return Err(UartError::UnsupportedType),
        };
        
        let mut driver = UartDriver {
            base_addr: config.uart_base,
            uart_type,
        };
        
        driver.hardware_init()?;
        Ok(driver)
    }
    
    fn probe(config: &DeviceConfig) -> bool {
        matches!(config.uart_type, "pl011" | "ns16550a")
    }
}

impl UartDriver {
    fn hardware_init(&mut self) -> Result<(), UartError> {
        match self.uart_type {
            UartType::Pl011 => self.init_pl011(),
            UartType::Ns16550a => self.init_ns16550a(),
        }
    }
    
    fn init_pl011(&mut self) -> Result<(), UartError> {
        // PL011 UART initialization
        unsafe {
            // Disable UART
            ptr::write_volatile((self.base_addr + 0x30) as *mut u32, 0);
            
            // Set baud rate (assuming 24MHz clock, 115200 baud)
            ptr::write_volatile((self.base_addr + 0x24) as *mut u32, 0x0d); // IBRD
            ptr::write_volatile((self.base_addr + 0x28) as *mut u32, 0x00); // FBRD
            
            // Set line control: 8N1, enable FIFO
            ptr::write_volatile((self.base_addr + 0x2c) as *mut u32, 0x70);
            
            // Enable UART, TX, RX
            ptr::write_volatile((self.base_addr + 0x30) as *mut u32, 0x301);
        }
        Ok(())
    }
    
    fn init_ns16550a(&mut self) -> Result<(), UartError> {
        // NS16550A UART initialization
        unsafe {
            // Set DLAB to access divisor latches
            ptr::write_volatile((self.base_addr + 3) as *mut u8, 0x80);
            
            // Set divisor for 115200 baud (assuming 10MHz clock)
            ptr::write_volatile((self.base_addr + 0) as *mut u8, 5);  // DLL
            ptr::write_volatile((self.base_addr + 1) as *mut u8, 0);  // DLH
            
            // Clear DLAB and set 8N1
            ptr::write_volatile((self.base_addr + 3) as *mut u8, 0x03);
            
            // Enable FIFOs
            ptr::write_volatile((self.base_addr + 2) as *mut u8, 0x01);
            
            // No interrupts
            ptr::write_volatile((self.base_addr + 1) as *mut u8, 0x00);
            
            // Set RTS and DTR
            ptr::write_volatile((self.base_addr + 4) as *mut u8, 0x03);
        }
        Ok(())
    }
    
    pub fn write_char(&self, c: u8) {
        match self.uart_type {
            UartType::Pl011 => self.pl011_write_char(c),
            UartType::Ns16550a => self.ns16550a_write_char(c),
        }
    }
    
    pub fn write_str(&self, s: &str) {
        for byte in s.bytes() {
            self.write_char(byte);
        }
    }
    
    pub fn read_char(&self) -> Option<u8> {
        match self.uart_type {
            UartType::Pl011 => self.pl011_read_char(),
            UartType::Ns16550a => self.ns16550a_read_char(),
        }
    }
    
    pub fn data_available(&self) -> bool {
        match self.uart_type {
            UartType::Pl011 => self.pl011_data_available(),
            UartType::Ns16550a => self.ns16550a_data_available(),
        }
    }
    
    // PL011 specific methods
    fn pl011_write_char(&self, c: u8) {
        unsafe {
            // Wait for TX FIFO not full
            while (ptr::read_volatile((self.base_addr + 0x18) as *const u32) & 0x20) != 0 {}
            ptr::write_volatile(self.base_addr as *mut u32, c as u32);
        }
    }
    
    fn pl011_read_char(&self) -> Option<u8> {
        unsafe {
            if (ptr::read_volatile((self.base_addr + 0x18) as *const u32) & 0x10) == 0 {
                Some((ptr::read_volatile(self.base_addr as *const u32) & 0xFF) as u8)
            } else {
                None
            }
        }
    }
    
    fn pl011_data_available(&self) -> bool {
        unsafe {
            (ptr::read_volatile((self.base_addr + 0x18) as *const u32) & 0x10) == 0
        }
    }
    
    // NS16550A specific methods
    fn ns16550a_write_char(&self, c: u8) {
        unsafe {
            // Wait for transmitter holding register empty
            while (ptr::read_volatile((self.base_addr + 5) as *const u8) & 0x20) == 0 {}
            ptr::write_volatile(self.base_addr as *mut u8, c);
        }
    }
    
    fn ns16550a_read_char(&self) -> Option<u8> {
        unsafe {
            if (ptr::read_volatile((self.base_addr + 5) as *const u8) & 0x01) != 0 {
                Some(ptr::read_volatile(self.base_addr as *const u8))
            } else {
                None
            }
        }
    }
    
    fn ns16550a_data_available(&self) -> bool {
        unsafe {
            (ptr::read_volatile((self.base_addr + 5) as *const u8) & 0x01) != 0
        }
    }
}
