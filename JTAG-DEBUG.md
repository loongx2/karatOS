# JTAG Debugging Guide for ARM RTOS

This directory contains scripts and documentation for JTAG debugging the ARM RTOS kernel using QEMU emulation and real hardware.

## QEMU JTAG Debugging (Recommended for Development)

### Quick Start

1. **Terminal 1 - Start QEMU with JTAG server:**
   ```bash
   ./qemu-jtag-debug.sh
   ```
   This will:
   - Build the kernel with debug symbols
   - Start QEMU with JTAG server on port 1234
   - Halt the CPU at startup waiting for GDB

2. **Terminal 2 - Connect GDB debugger:**
   ```bash
   ./debug-rust-gdb.sh
   ```
   This will:
   - Connect to QEMU's JTAG server
   - Load debug symbols
   - Set breakpoints on main() and panic handlers
   - Start debugging session

### Debugging Workflow

```bash
# 1. Start QEMU debugging server
./qemu-jtag-debug.sh

# 2. In another terminal, connect GDB
./debug-rust-gdb.sh

# 3. GDB Commands for RTOS debugging
(gdb) break scheduler::schedule     # Break in scheduler
(gdb) break process_uart_commands   # Break in UART handler
(gdb) break simulate_async_task_work # Break in task execution
(gdb) continue                      # Continue execution
(gdb) step                          # Step into function
(gdb) next                          # Step over function
(gdb) info registers                # Show CPU registers
(gdb) bt                           # Show call stack
(gdb) print variable_name          # Print variable value
(gdb) x/10x 0x20000000             # Examine memory (RAM start)
```

## Real Hardware JTAG Debugging

### Prerequisites

Install debugging tools:
```bash
# Ubuntu/Debian
sudo apt install openocd gdb-multiarch

# macOS
brew install openocd arm-none-eabi-gdb
```

### Hardware Setup

1. **Connect JTAG/SWD debugger** (ST-Link, J-Link, etc.) to your ARM board
2. **Update OpenOCD configuration** in `debug-openocd.sh` for your hardware:
   ```bash
   # Example for STM32F103 with ST-Link
   -f interface/stlink.cfg \
   -f target/stm32f1x.cfg \
   
   # Example for STM32F4 with J-Link
   -f interface/jlink.cfg \
   -f target/stm32f4x.cfg \
   ```

3. **Start OpenOCD JTAG server:**
   ```bash
   ./debug-openocd.sh
   ```

4. **Connect GDB** (modify port 1234 → 3333 in debug scripts)

## JTAG Features Enabled

### QEMU Debugging Features
- **CPU Halt Control**: Start/stop/reset CPU execution
- **Memory Access**: Read/write RAM, Flash, and peripherals  
- **Register Inspection**: View ARM Cortex-M registers
- **Breakpoints**: Software and hardware breakpoints
- **Single Stepping**: Step through code instruction by instruction
- **Call Stack**: Full function call trace with debug symbols

### RTOS-Specific Debugging
- **Task Scheduling**: Step through cooperative task switching
- **Event Processing**: Debug priority-based event handling
- **UART Interface**: Debug command processing in real-time
- **Memory Management**: Inspect static buffers and heapless collections
- **Async Execution**: Trace async task state machines

## GDB Command Reference

### Essential Commands
```bash
# Connection and Control
target remote localhost:1234    # Connect to QEMU JTAG
monitor reset halt              # Reset and halt CPU
load                           # Load program to target
continue / c                   # Continue execution
interrupt / Ctrl+C             # Break execution

# Breakpoints
break function_name            # Break at function
break file.rs:123             # Break at line number
break *0x08000100             # Break at address
info breakpoints              # List breakpoints
delete 1                      # Delete breakpoint 1

# Execution Control
step / s                      # Step into function
next / n                      # Step over function
finish                        # Run until function returns
until                         # Run until next line

# Memory and Variables
print variable               # Print variable value
print/x variable            # Print in hexadecimal
x/10x 0x20000000           # Examine 10 words at address
info registers             # Show all registers
info locals               # Show local variables

# Call Stack
bt                        # Show call stack
up                        # Move up call stack
down                      # Move down call stack
frame 2                   # Jump to frame 2
```

### RTOS-Specific Debugging

```bash
# Debug Task Scheduling
break scheduler::schedule
break scheduler::add_task
break scheduler::post_event_with_priority

# Debug UART Interface  
break process_uart_commands
break handle_uart_command
break uart::process_byte

# Debug Logging System
break logger::Logger::log
break logger::Logger::get_last_lines

# Debug Task Execution
break simulate_async_task_work
print TASK_WORK_COUNTER
print ITERATION_COUNT
```

## Debugging Examples

### Example 1: Debug Task Scheduling
```bash
(gdb) break scheduler::schedule
(gdb) continue
# CPU halts when scheduler runs
(gdb) step  # Step through scheduler logic
(gdb) print ready_queue  # Inspect task queue
```

### Example 2: Debug UART Commands
```bash
(gdb) break handle_uart_command
(gdb) continue
# Type 'status' in UART interface
# GDB breaks when command is processed
(gdb) print command  # See parsed command
(gdb) step  # Step through command handling
```

### Example 3: Memory Inspection
```bash
(gdb) x/100x 0x20000000  # Examine RAM
(gdb) x/s &LOG_BUFFER    # Examine log buffer
(gdb) print/x UART_INTERFACE  # Check UART state
```

## Troubleshooting

### Common Issues

**QEMU won't start with JTAG:**
- Check port 1234 is not in use: `netstat -an | grep 1234`
- Kill existing QEMU: `pkill qemu-system-arm`

**GDB can't connect:**
- Ensure QEMU is running with `-gdb tcp::1234 -S`
- Check firewall settings for localhost:1234

**No debug symbols:**
- Ensure building in debug mode (not --release)
- Check debug symbols: `file target/thumbv7m-none-eabi/debug/kernel`

**ARM GDB not found:**
- Install: `sudo apt install gdb-multiarch arm-none-eabi-gdb`
- Or use: `gdb-multiarch` instead of `arm-none-eabi-gdb`

### Hardware Debugging Issues

**OpenOCD connection failed:**
- Check debugger connection and power
- Verify correct interface/target configuration
- Try lower adapter speed: `-c "adapter speed 100"`

**Target not responding:**
- Check target power and reset
- Try different transport: `hla_swd` vs `jtag`
- Verify target configuration matches your hardware

## Files

- `qemu-jtag-debug.sh` - Start QEMU with JTAG debugging
- `debug-rust-gdb.sh` - Connect GDB with Rust support  
- `debug-gdb.sh` - Alternative GDB connection script
- `debug-openocd.sh` - OpenOCD setup for real hardware
- `JTAG-DEBUG.md` - This documentation

## Next Steps

After setting up JTAG debugging, you can:
1. Set breakpoints in critical RTOS functions
2. Step through task scheduling algorithms
3. Debug UART command processing in real-time
4. Inspect memory layout and static buffers
5. Profile task execution timing
6. Debug async event handling logic
