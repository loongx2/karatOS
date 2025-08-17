#!/bin/bash
# QEMU JTAG debugging with automatic start (no halt)

echo "=== ARM RTOS JTAG Debugging (Auto-Start) ==="

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
echo ""
echo "Starting QEMU with JTAG debugging (auto-start)..."
echo "GDB server available on localhost:1234"
echo "CPU will start running immediately"
echo "Use './debug-rust-gdb.sh' to connect and break execution"
echo ""

# QEMU with JTAG debugging but no halt (-S removed)
qemu-system-arm \
    -M lm3s6965evb \
    -nographic \
    -semihosting-config enable=on,target=native \
    -serial mon:stdio \
    -gdb tcp::1234 \
    -kernel ../target/thumbv7m-none-eabi/debug/kernel

echo ""
echo "QEMU debugging session ended."
