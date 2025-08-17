#!/bin/bash
# Fixed GDB debugging script for ARM RTOS kernel

echo "=== ARM RTOS GDB Debugging (Fixed) ==="

# Check if QEMU is running
if ! nc -z localhost 1234 2>/dev/null; then
    echo "❌ QEMU JTAG server not found on localhost:1234"
    echo "Please start QEMU with: ./qemu-jtag-debug.sh"
    exit 1
fi

echo "✅ QEMU JTAG server detected"

# Use available GDB
if command -v gdb-multiarch &> /dev/null; then
    GDB_CMD="gdb-multiarch"
else
    GDB_CMD="gdb"
fi

echo "Using: $GDB_CMD"
echo ""

# Fixed GDB session with proper ARM architecture
$GDB_CMD \
    -ex "set confirm off" \
    -ex "set architecture armv7" \
    -ex "set endian little" \
    -ex "file target/thumbv7m-none-eabi/debug/kernel" \
    -ex "target remote localhost:1234" \
    -ex "monitor reset" \
    -ex "monitor halt" \
    -ex "load" \
    -ex "monitor reset" \
    -ex "monitor halt" \
    -ex "break main" \
    -ex "break rust_begin_unwind" \
    -ex "set print pretty on" \
    -ex "continue" \
    -ex "echo \n=== GDB Connected and Running ===\n" \
    -ex "echo Available commands:\n" \
    -ex "echo   step (s)     - Step into function\n" \
    -ex "echo   next (n)     - Step over function\n" \
    -ex "echo   continue (c) - Continue execution\n" \
    -ex "echo   break func   - Set breakpoint\n" \
    -ex "echo   info locals  - Show variables\n" \
    -ex "echo   bt           - Show call stack\n" \
    -ex "echo ===============================\n"

echo ""
echo "GDB debugging session ended."
