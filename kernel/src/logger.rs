// Circular log buffer for capturing system debug output
// Stores up to 100 log lines in static memory with rollover (reduced for memory constraints)

use heapless::{String, Vec};

const MAX_LOG_LINES: usize = 100;  // Reduced from 1000
const MAX_LINE_LENGTH: usize = 64;  // Reduced from 128
const STATUS_SNAPSHOT_LINES: usize = 50;  // Reduced from 100

type LogLine = String<MAX_LINE_LENGTH>;
type LogBuffer = Vec<LogLine, MAX_LOG_LINES>;

// Static circular log buffer
static mut LOG_BUFFER: LogBuffer = Vec::new();
static mut LOG_INDEX: usize = 0;
static mut TOTAL_LINES: usize = 0;

pub struct Logger;

impl Logger {
    /// Add a new log line to the circular buffer
    #[allow(static_mut_refs)]
    pub fn log(message: &str) {
        unsafe {
            let mut log_line = LogLine::new();
            let _ = log_line.push_str(message);
            
            if LOG_BUFFER.len() < MAX_LOG_LINES {
                // Buffer not full yet, just push
                let _ = LOG_BUFFER.push(log_line);
            } else {
                // Buffer is full, overwrite at current index (circular)
                LOG_BUFFER[LOG_INDEX] = log_line;
            }
            
            // Update circular index
            LOG_INDEX = (LOG_INDEX + 1) % MAX_LOG_LINES;
            TOTAL_LINES += 1;
        }
    }
    
    /// Get the last N lines for status command
    #[allow(static_mut_refs)]
    pub fn get_last_lines(count: usize) -> Vec<LogLine, STATUS_SNAPSHOT_LINES> {
        let mut result = Vec::new();
        
        unsafe {
            let buffer_size = LOG_BUFFER.len();
            if buffer_size == 0 {
                return result;
            }
            
            let lines_to_get = count.min(buffer_size).min(STATUS_SNAPSHOT_LINES);
            
            if buffer_size < MAX_LOG_LINES {
                // Buffer not full yet, get from end
                let start_idx = buffer_size.saturating_sub(lines_to_get);
                for i in start_idx..buffer_size {
                    if result.push(LOG_BUFFER[i].clone()).is_err() {
                        break;
                    }
                }
            } else {
                // Buffer is full, get from circular position
                let start_idx = if LOG_INDEX >= lines_to_get {
                    LOG_INDEX - lines_to_get
                } else {
                    MAX_LOG_LINES - (lines_to_get - LOG_INDEX)
                };
                
                for i in 0..lines_to_get {
                    let idx = (start_idx + i) % MAX_LOG_LINES;
                    if result.push(LOG_BUFFER[idx].clone()).is_err() {
                        break;
                    }
                }
            }
        }
        
        result
    }
    
    /// Get statistics about the log buffer
    #[allow(static_mut_refs)]
    pub fn get_stats() -> (usize, usize, usize) {
        unsafe {
            (LOG_BUFFER.len(), TOTAL_LINES, LOG_INDEX)
        }
    }
    
    /// Clear the log buffer
    #[allow(static_mut_refs)]
    pub fn clear() {
        unsafe {
            LOG_BUFFER.clear();
            LOG_INDEX = 0;
            TOTAL_LINES = 0;
        }
    }
}

/// Macro for silent logging (replaces arch_println for debug output)
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        {
            use heapless::String;
            let mut msg = String::<64>::new();  // Reduced from 128
            use core::fmt::Write;
            let _ = write!(msg, $($arg)*);
            crate::logger::Logger::log(msg.as_str());
        }
    };
}

/// Macro for visible output (still goes to terminal)
#[macro_export]
macro_rules! log_visible {
    ($($arg:tt)*) => {
        {
            // Also log to buffer
            use heapless::String;
            let mut msg = String::<64>::new();  // Reduced from 128
            use core::fmt::Write;
            let _ = write!(msg, $($arg)*);
            crate::logger::Logger::log(msg.as_str());
            
            // And print to terminal
            crate::arch::arch_println(&msg);
        }
    };
}
