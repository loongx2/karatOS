#!/bin/bash
# Simple interactive GDB session for JTAG debugging

echo "=== ARM RTOS GDB Interactive Session ==="

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
echo "📋 IMPORTANT COMMANDS:"
echo "   continue (c) - Start/continue execution (NOT 'run'!)"
echo "   step (s)     - Step into function"
echo "   next (n)     - Step over function"
echo "   break main   - Set breakpoint at main"
echo "   info locals  - Show variables"
echo "   bt           - Show call stack"
echo "   quit         - Exit GDB"
echo ""
echo "⚠️  DO NOT use 'run' or 'start' - use 'continue' instead!"
echo ""

# Start interactive GDB session
$GDB_CMD \
    -ex "set confirm off" \
    -ex "set architecture armv7" \
    -ex "set endian little" \
    -ex "file target/thumbv7m-none-eabi/debug/kernel" \
    -ex "target remote localhost:1234" \
    -ex "monitor reset halt" \
    -ex "load" \
    -ex "break main" \
    -ex "break rust_begin_unwind" \
    -ex "set print pretty on" \
    -ex "echo \n=== GDB Connected to QEMU JTAG ===\n" \
    -ex "echo Type 'continue' to start execution\n" \
    -ex "echo Type 'help' for GDB help\n" \
    -ex "echo ================================\n"

echo ""
echo "GDB session ended."
