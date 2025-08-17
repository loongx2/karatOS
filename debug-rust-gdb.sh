#!/bin/bash
# Alternative GDB debugging using Rust-specific tooling

echo "=== ARM RTOS Rust GDB Debugging ==="

# Check if QEMU is running with JTAG
if ! nc -z localhost 1234 2>/dev/null; then
    echo "Error: QEMU JTAG server not found on localhost:1234"
    echo "Please start QEMU with: ./qemu-jtag-debug.sh"
    exit 1
fi

echo "Connecting to QEMU JTAG server with Rust-aware GDB..."
echo ""

# Use gdb (fallback if gdb-multiarch not available)
if command -v gdb-multiarch &> /dev/null; then
    GDB_CMD="gdb-multiarch"
elif command -v arm-none-eabi-gdb &> /dev/null; then
    GDB_CMD="arm-none-eabi-gdb"
else
    GDB_CMD="gdb"
fi

echo "Using GDB: $GDB_CMD"

$GDB_CMD \
    -ex "set architecture armv7" \
    -ex "set endian little" \
    -ex "file target/thumbv7m-none-eabi/debug/kernel" \
    -ex "target remote localhost:1234" \
    -ex "monitor reset halt" \
    -ex "load" \
    -ex "break main" \
    -ex "break rust_begin_unwind" \
    -ex "set print pretty on" \
    -ex "set print array-indexes on" \
    -ex "info registers" \
    -ex "continue"

echo ""
echo "GDB debugging session ended."
