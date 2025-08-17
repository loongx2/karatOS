#!/bin/bash
# Simple GDB connection test to verify JTAG is working

echo "=== Testing JTAG Connection ==="

# Check if QEMU JTAG server is running
if ! nc -z localhost 1234 2>/dev/null; then
    echo "❌ QEMU JTAG server not found on localhost:1234"
    echo "Please start QEMU with: ./qemu-jtag-debug.sh (in another terminal)"
    exit 1
fi

echo "✅ QEMU JTAG server detected on localhost:1234"
echo ""

# Use available GDB
if command -v gdb-multiarch &> /dev/null; then
    GDB_CMD="gdb-multiarch"
elif command -v arm-none-eabi-gdb &> /dev/null; then
    GDB_CMD="arm-none-eabi-gdb"
else
    GDB_CMD="gdb"
fi

echo "Using GDB: $GDB_CMD"
echo "Connecting for a quick test..."
echo ""

# Quick connection test
$GDB_CMD \
    -batch \
    -ex "set architecture arm" \
    -ex "target remote localhost:1234" \
    -ex "info registers" \
    -ex "detach" \
    -ex "quit"

echo ""
echo "✅ JTAG connection test completed!"
echo "You can now use ./debug-rust-gdb.sh for interactive debugging"
