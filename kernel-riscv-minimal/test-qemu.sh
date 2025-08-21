#!/bin/bash

# Simple RISC-V QEMU test script
set -e

KERNEL_PATH="target/riscv32imac-unknown-none-elf/debug/kernel-riscv-minimal"

echo "=== Testing Minimal RISC-V Kernel ==="

if [ ! -f "$KERNEL_PATH" ]; then
    echo "Error: Kernel not found. Building..."
    cargo build --target riscv32imac-unknown-none-elf
fi

echo "Launching QEMU with timeout..."
echo "Note: QEMU will auto-exit in 10 seconds if kernel doesn't exit properly"

# Try different QEMU configurations to get UART output working
timeout 10s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -smp 1 \
    -m 128M \
    -nographic \
    -bios none \
    -serial stdio \
    -kernel "$KERNEL_PATH"

EXIT_CODE=$?
echo ""
echo "QEMU exited with code: $EXIT_CODE"
if [ $EXIT_CODE -eq 124 ]; then
    echo "QEMU timed out - kernel may be running but not exiting properly"
elif [ $EXIT_CODE -eq 0 ]; then
    echo "QEMU exited successfully - kernel test completed!"
else
    echo "QEMU exited with error code $EXIT_CODE"
fi
