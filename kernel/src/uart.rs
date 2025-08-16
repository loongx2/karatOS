//! UART Interface for QEMU Virtualization
//! 
//! Provides a simple UART interface for receiving and processing commands
//! in QEMU environment. Supports basic commands for system control.

use heapless::{String, Vec};

/// Maximum command length
const MAX_COMMAND_LEN: usize = 32;

/// Command buffer size
const COMMAND_BUFFER_SIZE: usize = 64;

/// Available UART commands
#[derive(Debug, Clone, PartialEq)]
pub enum UartCommand {
    Status,        // Show system status
    Exit,          // Halt and exit system
    Restart,       // Reboot system
    Help,          // Show available commands
    Unknown(String<MAX_COMMAND_LEN>), // Unknown command
}

/// UART command parser and handler
pub struct UartInterface {
    command_buffer: String<MAX_COMMAND_LEN>,
    input_buffer: Vec<u8, COMMAND_BUFFER_SIZE>,
}

impl UartInterface {
    /// Create new UART interface
    pub const fn new() -> Self {
        Self {
            command_buffer: String::new(),
            input_buffer: Vec::new(),
        }
    }
    
    /// Process incoming byte and return command if complete
    pub fn process_byte(&mut self, byte: u8) -> Option<UartCommand> {
        match byte {
            // Newline or carriage return - process command
            b'\n' | b'\r' => {
                if !self.input_buffer.is_empty() {
                    // Convert buffer to string
                    if let Ok(cmd_str) = core::str::from_utf8(&self.input_buffer) {
                        self.command_buffer.clear();
                        let _ = self.command_buffer.push_str(cmd_str.trim());
                        let command = self.parse_command();
                        self.input_buffer.clear();
                        Some(command)
                    } else {
                        self.input_buffer.clear();
                        None
                    }
                } else {
                    None
                }
            },
            // Backspace - remove last character
            b'\x08' | b'\x7f' => {
                self.input_buffer.pop();
                None
            },
            // Printable ASCII characters
            b' '..=b'~' => {
                if self.input_buffer.len() < COMMAND_BUFFER_SIZE - 1 {
                    let _ = self.input_buffer.push(byte);
                }
                None
            },
            // Ignore other characters
            _ => None,
        }
    }
    
    /// Parse command from buffer
    fn parse_command(&self) -> UartCommand {
        let cmd = self.command_buffer.as_str();
        // Manual lowercase conversion for no_std
        let mut lowercase_cmd = String::<MAX_COMMAND_LEN>::new();
        for c in cmd.chars() {
            if let Some(lower_c) = c.to_lowercase().next() {
                let _ = lowercase_cmd.push(lower_c);
            }
        }
        
        match lowercase_cmd.as_str() {
            "status" => UartCommand::Status,
            "exit" => UartCommand::Exit,
            "restart" | "reboot" => UartCommand::Restart,
            "help" | "?" => UartCommand::Help,
            "" => UartCommand::Help, // Empty command shows help
            _ => {
                let mut unknown = String::new();
                let _ = unknown.push_str(&lowercase_cmd);
                UartCommand::Unknown(unknown)
            }
        }
    }
    
    /// Get current input buffer as string (for echo)
    #[allow(dead_code)]
    pub fn get_current_input(&self) -> &str {
        core::str::from_utf8(&self.input_buffer).unwrap_or("")
    }
    
    /// Clear input buffer
    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
        self.command_buffer.clear();
    }
}

/// UART command handler responses
pub struct UartResponses;

impl UartResponses {
    /// Get response for status command
    pub fn status_response() -> &'static str {
        "RTOS Status:\n\
         - Kernel: Async Event-Driven RTOS v0.1.0\n\
         - Scheduler: Priority-based Cooperative Multitasking\n\
         - Architecture: ARM Cortex-M / RISC-V\n\
         - Tasks: Active and running\n\
         - Events: Processing normally\n\
         - UART: Interface active\n\
         Ready.\n"
    }
    
    /// Get response for help command
    pub fn help_response() -> &'static str {
        "Available Commands:\n\
         - status    : Show system status\n\
         - exit      : Halt and exit system\n\
         - restart   : Reboot system\n\
         - help      : Show this help message\n\
         \n\
         Type command and press Enter.\n"
    }
    
    /// Get response for exit command
    pub fn exit_response() -> &'static str {
        "System shutdown initiated...\n\
         Stopping all tasks...\n\
         Halting system.\n"
    }
    
    /// Get response for restart command
    pub fn restart_response() -> &'static str {
        "System restart initiated...\n\
         Stopping all tasks...\n\
         Rebooting system...\n"
    }
    
    /// Get response for unknown command
    pub fn unknown_response(cmd: &str) -> String<128> {
        let mut response = String::new();
        let _ = response.push_str("Unknown command: '");
        let _ = response.push_str(cmd);
        let _ = response.push_str("'\nType 'help' for available commands.\n");
        response
    }
    
    /// Get welcome message
    pub fn welcome_message() -> &'static str {
        "\n=== UART Interface Active ===\n\
         Async Event-Driven RTOS Kernel\n\
         Type 'help' for available commands.\n\
         UART> "
    }
    
    /// Get command prompt
    pub fn prompt() -> &'static str {
        "UART> "
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_parsing() {
        let mut uart = UartInterface::new();
        
        // Test status command
        let cmd = uart.process_byte(b's');
        assert_eq!(cmd, None);
        let cmd = uart.process_byte(b't');
        assert_eq!(cmd, None);
        let cmd = uart.process_byte(b'a');
        assert_eq!(cmd, None);
        let cmd = uart.process_byte(b't');
        assert_eq!(cmd, None);
        let cmd = uart.process_byte(b'u');
        assert_eq!(cmd, None);
        let cmd = uart.process_byte(b's');
        assert_eq!(cmd, None);
        let cmd = uart.process_byte(b'\n');
        assert_eq!(cmd, Some(UartCommand::Status));
        
        // Test exit command
        for byte in b"exit\r" {
            uart.process_byte(*byte);
        }
        let cmd = uart.process_byte(b'\n');
        assert_eq!(cmd, Some(UartCommand::Exit));
    }
    
    #[test]
    fn test_backspace() {
        let mut uart = UartInterface::new();
        
        uart.process_byte(b'h');
        uart.process_byte(b'e');
        uart.process_byte(b'l');
        uart.process_byte(b'\x08'); // backspace
        uart.process_byte(b'l');
        uart.process_byte(b'p');
        let cmd = uart.process_byte(b'\n');
        assert_eq!(cmd, Some(UartCommand::Help));
    }
}
