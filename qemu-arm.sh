#!/bin/bash
# Build and run ARM kernel on QEMU (Optimized)

set -e  # Exit on any error

echo "=== karatOS ARM QEMU Runner ==="

# Check if binary exists and is up-to-date
TARGET_DIR="target/thumbv7m-none-eabi/debug"
KERNEL_BINARY="$TARGET_DIR/kernel"

# Check if we need to rebuild
if [ ! -f "$KERNEL_BINARY" ] || [ "kernel/src/main.rs" -nt "$KERNEL_BINARY" ]; then
    echo "Building ARM kernel..."
    ./build.sh arm
else
    echo "Using existing ARM kernel binary"
fi

# Verify binary exists
if [ ! -f "$KERNEL_BINARY" ]; then
    echo "Error: ARM kernel binary not found at $KERNEL_BINARY"
    exit 1
fi

echo "Starting ARM QEMU with live task scheduling..."
echo "Press Ctrl+C to stop"
echo ""

# Run in QEMU with semihosting and UART enabled
qemu-system-arm \
    -M lm3s6965evb \
    -nographic \
    -semihosting-config enable=on,target=native \
    -serial mon:stdio \
    -kernel "$KERNEL_BINARY"
