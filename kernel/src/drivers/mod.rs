//! Hardware driver modules
//! Architecture-agnostic drivers for various hardware components

pub mod uart {
    //! Simple UART driver for debugging output
    
    /// Initialize UART driver
    pub fn init() {
        // UART initialization will be handled by architecture-specific code
        crate::arch::early_println("UART driver initialized");
    }
    
    /// Print a string to UART
    pub fn print(msg: &str) {
        crate::arch::early_println(msg);
    }
}
