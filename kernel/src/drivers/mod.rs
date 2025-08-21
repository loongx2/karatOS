//! Device Driver Framework
//! Device tree driven driver initialization and management

pub mod uart;
pub mod timer;

/// Device configuration structure
/// Represents device tree information for hardware components
#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub uart_base: usize,
    pub uart_type: &'static str,
    pub timer_base: Option<usize>,
    pub memory_base: usize,
    pub memory_size: usize,
}

/// Driver trait for all hardware drivers
pub trait Driver {
    type Error;
    
    fn init(config: &DeviceConfig) -> Result<Self, Self::Error>
    where 
        Self: Sized;
    
    fn probe(config: &DeviceConfig) -> bool;
}

/// Driver manager for initializing all drivers based on device configuration
pub struct DriverManager {
    uart_driver: Option<uart::UartDriver>,
    timer_driver: Option<timer::TimerDriver>,
}

impl DriverManager {
    pub fn new() -> Self {
        Self {
            uart_driver: None,
            timer_driver: None,
        }
    }
    
    /// Initialize all drivers based on device configuration
    pub fn init_drivers(&mut self, config: &DeviceConfig) -> Result<(), &'static str> {
        // Initialize UART driver
        if uart::UartDriver::probe(config) {
            match uart::UartDriver::init(config) {
                Ok(driver) => {
                    self.uart_driver = Some(driver);
                    crate::arch::arch_println("UART driver initialized");
                }
                Err(_) => return Err("Failed to initialize UART driver"),
            }
        }
        
        // Initialize Timer driver if available
        if config.timer_base.is_some() && timer::TimerDriver::probe(config) {
            match timer::TimerDriver::init(config) {
                Ok(driver) => {
                    self.timer_driver = Some(driver);
                    crate::arch::arch_println("Timer driver initialized");
                }
                Err(_) => return Err("Failed to initialize Timer driver"),
            }
        }
        
        Ok(())
    }
    
    /// Get UART driver instance
    pub fn uart(&self) -> Option<&uart::UartDriver> {
        self.uart_driver.as_ref()
    }
    
    /// Get mutable UART driver instance
    pub fn uart_mut(&mut self) -> Option<&mut uart::UartDriver> {
        self.uart_driver.as_mut()
    }
    
    /// Get Timer driver instance
    pub fn timer(&self) -> Option<&timer::TimerDriver> {
        self.timer_driver.as_ref()
    }
}
