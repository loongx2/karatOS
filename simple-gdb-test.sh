#!/bin/bash
# Simple GDB test for JTAG debugging

echo "=== Simple GDB JTAG Test ==="

# Check if QEMU is running
if ! nc -z localhost 1234 2>/dev/null; then
    echo "❌ No QEMU JTAG server on localhost:1234"
    echo "Start QEMU first with: ./qemu-jtag-debug.sh"
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

# Simple connection test
$GDB_CMD \
    -ex "set architecture arm" \
    -ex "set endian little" \
    -ex "file target/thumbv7m-none-eabi/debug/kernel" \
    -ex "target remote localhost:1234" \
    -ex "info registers" \
    -ex "break main" \
    -ex "continue" \
    -ex "backtrace" \
    -ex "quit"
