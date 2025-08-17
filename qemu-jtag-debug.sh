#!/bin/bash
# Build and run ARM kernel on QEMU with JTAG debugging support

echo "=== ARM RTOS JTAG Debugging Setup ==="

# Ensure ARM target is installed
rustup target add thumbv7m-none-eabi

# Build the kernel with ARM features and debug symbols
cd kernel
echo "Building kernel with debug symbols..."
cargo build --bin kernel --target thumbv7m-none-eabi --features arm

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo "Kernel built successfully with debug symbols"
echo "Binary location: ../target/thumbv7m-none-eabi/debug/kernel"

# Run QEMU with JTAG debugging enabled
echo ""
echo "Starting QEMU with JTAG debugging..."
echo "GDB server will be available on localhost:1234"
echo "Use './debug-gdb.sh' in another terminal to connect GDB"
echo ""
echo "QEMU Debug Options:"
echo "- JTAG debugging: -gdb tcp::1234 -S (halt on startup)"
echo "- Monitor console: Ctrl+A, C to access QEMU monitor"
echo "- Exit QEMU: Ctrl+A, X"
echo ""

# QEMU with JTAG debugging support
qemu-system-arm \
    -M lm3s6965evb \
    -nographic \
    -semihosting-config enable=on,target=native \
    -serial mon:stdio \
    -gdb tcp::1234 \
    -S \
    -kernel ../target/thumbv7m-none-eabi/debug/kernel

echo ""
echo "QEMU debugging session ended."
