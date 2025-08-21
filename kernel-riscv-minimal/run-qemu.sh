#!/bin/bash

# Test RISC-V kernel with proper QEMU configuration
set -e

KERNEL_PATH="target/riscv32imac-unknown-none-elf/debug/kernel-riscv-minimal"

echo "=== Testing RISC-V Kernel ==="

if [ ! -f "$KERNEL_PATH" ]; then
    echo "Error: Kernel not found. Building..."
    cargo build --target riscv32imac-unknown-none-elf
fi

echo "Starting QEMU with RISC-V kernel..."
echo "Kernel should output 'HELLO' to serial console"
echo "Press Ctrl+A X to exit QEMU"

qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -smp 1 \
    -m 128M \
    -nographic \
    -bios none \
    -kernel "$KERNEL_PATH"
