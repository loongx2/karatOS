# JTAG Debugging Quick Start Guide

## The Problem You Encountered

The error occurs when you use `run` or `start` in GDB instead of `continue`. Here's why:

- ❌ **WRONG**: `run` tries to execute the ARM binary on your x86_64 host
- ✅ **CORRECT**: `continue` resumes the program already loaded in QEMU

## Step-by-Step JTAG Debugging

### Step 1: Start QEMU with JTAG
```bash
./qemu-jtag-debug.sh
```
This starts QEMU and waits for GDB connection.

### Step 2: Connect GDB (Interactive)
```bash
./debug-interactive.sh
```
This connects GDB to QEMU and gives you an interactive prompt.

### Step 3: Debug Commands (In GDB)
```bash
(gdb) continue          # ✅ Start execution (NOT 'run'!)
(gdb) break main        # Set breakpoint
(gdb) continue          # Continue to breakpoint
(gdb) step              # Step into function
(gdb) next              # Step over function
(gdb) info locals       # Show variables
(gdb) bt                # Show call stack
(gdb) quit              # Exit GDB
```

## Common Mistakes

### ❌ DON'T DO THIS:
```bash
(gdb) run               # Will fail with "Exec format error"
(gdb) start             # Will fail with "Exec format error"
```

### ✅ DO THIS INSTEAD:
```bash
(gdb) continue          # Resume execution in QEMU
(gdb) c                 # Short form of continue
```

## Available Scripts

1. **`./qemu-jtag-debug.sh`** - Start QEMU with JTAG server
2. **`./debug-interactive.sh`** - Connect GDB interactively (recommended)
3. **`./debug-fixed.sh`** - Automated debugging session
4. **`./debug-rust-gdb.sh`** - Alternative GDB connection

## Debugging Workflow Example

```bash
# Terminal 1: Start QEMU
./qemu-jtag-debug.sh

# Terminal 2: Connect GDB
./debug-interactive.sh

# In GDB:
(gdb) continue                           # Start execution
(gdb) break process_uart_commands        # Break in UART handler
(gdb) continue                           # Continue to breakpoint
(gdb) step                               # Step through code
(gdb) print UART_INTERFACE              # Examine variables
(gdb) bt                                 # Show call stack
(gdb) quit                               # Exit
```

## Why This Works

- **QEMU**: Emulates ARM CPU and provides JTAG server on port 1234
- **GDB**: Connects to QEMU as a remote target
- **`continue`**: Resumes the ARM program running in QEMU
- **`run`**: Tries to execute ARM binary on host (fails!)

## Troubleshooting

**"Exec format error"**: You used `run` instead of `continue`
**"Connection refused"**: QEMU not running or wrong port
**"No symbol table"**: Binary not loaded, use `load` command

Remember: **Always use `continue`, never `run`** when debugging with QEMU!
