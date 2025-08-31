#!/bin/bash
# Build and run RISC-V kernel on QEMU (Optimized)

set -e  # Exit on any error

echo "=== karatOS RISC-V QEMU Runner ==="

# Check if binary exists and is up-to-date
TARGET_DIR="target/riscv32imac-unknown-none-elf/debug"
KERNEL_BINARY="$TARGET_DIR/kernel"

# Check if we need to rebuild
if [ ! -f "$KERNEL_BINARY" ] || [ "kernel/src/main.rs" -nt "$KERNEL_BINARY" ]; then
    echo "Building RISC-V kernel..."
    ./build.sh riscv
else
    echo "Using existing RISC-V kernel binary"
fi

# Verify binary exists
if [ ! -f "$KERNEL_BINARY" ]; then
    echo "Error: RISC-V kernel binary not found at $KERNEL_BINARY"
    exit 1
fi

echo "Starting RISC-V QEMU with live task scheduling..."
echo "Press Ctrl+C to stop"
echo ""

# Run in QEMU virt machine with UART output
qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -smp 1 \
    -m 128M \
    -nographic \
    -bios none \
    -serial mon:stdio \
    -kernel "$KERNEL_BINARY"
