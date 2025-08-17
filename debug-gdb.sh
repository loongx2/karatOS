#!/bin/bash
# GDB debugging script for ARM RTOS kernel

echo "=== ARM RTOS GDB Debugging ==="

# Check if QEMU is running with JTAG
if ! nc -z localhost 1234 2>/dev/null; then
    echo "Error: QEMU JTAG server not found on localhost:1234"
    echo "Please start QEMU with: ./qemu-jtag-debug.sh"
    exit 1
fi

echo "Connecting to QEMU JTAG server on localhost:1234..."
echo ""

# Start GDB with ARM cross-debugging support
arm-none-eabi-gdb \
    -ex "set architecture arm" \
    -ex "target remote localhost:1234" \
    -ex "monitor reset halt" \
    -ex "load" \
    -ex "break main" \
    -ex "break panic_handler" \
    -ex "continue" \
    target/thumbv7m-none-eabi/debug/kernel

echo ""
echo "GDB debugging session ended."
