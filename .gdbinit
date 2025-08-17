# GDB configuration for ARM RTOS debugging
# Place this in ~/.gdbinit or project .gdbinit

# ARM Cortex-M specific settings
set architecture armv7
set endian little

# Improve debugging experience
set print pretty on
set print array-indexes on
set print symbol-filename on
set print frame-arguments all

# Show more context when hitting breakpoints
set listsize 20

# Disable confirmation prompts
set confirm off

# Auto-load debugging scripts
set auto-load safe-path /

# Custom commands for RTOS debugging
define rtos-info
    printf "=== RTOS Debug Information ===\n"
    printf "Task Counters: "
    print TASK_WORK_COUNTER
    printf "Iteration Count: "
    print ITERATION_COUNT
    printf "Terminate Flag: "
    print TERMINATE_FLAG
    printf "Log Buffer Stats: "
    call logger::Logger::get_stats()
end

define uart-info
    printf "=== UART Interface State ===\n"
    printf "UART Interface: "
    print UART_INTERFACE
end

define scheduler-info
    printf "=== Scheduler Information ===\n"
    call scheduler::scheduler_stats()
end

# Set common breakpoints
define rtos-breakpoints
    break main
    break rust_begin_unwind
    break panic_handler
    break scheduler::schedule
    break process_uart_commands
    break handle_uart_command
    printf "RTOS breakpoints set\n"
end

# Connect to QEMU shortcut
define qemu-connect
    target remote localhost:1234
    monitor reset halt
    load
    printf "Connected to QEMU JTAG\n"
end

# Connect to OpenOCD shortcut  
define openocd-connect
    target remote localhost:3333
    monitor reset halt
    load
    printf "Connected to OpenOCD JTAG\n"
end

# Display banner
printf "\n=== ARM RTOS GDB Configuration Loaded ===\n"
printf "Custom commands available:\n"
printf "  rtos-info        - Show RTOS debug information\n"
printf "  uart-info        - Show UART interface state\n" 
printf "  scheduler-info   - Show scheduler statistics\n"
printf "  rtos-breakpoints - Set common RTOS breakpoints\n"
printf "  qemu-connect     - Connect to QEMU JTAG (port 1234)\n"
printf "  openocd-connect  - Connect to OpenOCD JTAG (port 3333)\n"
printf "==========================================\n\n"
