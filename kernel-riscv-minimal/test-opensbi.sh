#!/bin/bash

# Test with OpenSBI firmware
set -e

KERNEL_PATH="target/riscv32imac-unknown-none-elf/debug/kernel-riscv-minimal"

echo "=== Testing RISC-V Kernel with OpenSBI ==="

if [ ! -f "$KERNEL_PATH" ]; then
    echo "Error: Kernel not found. Building..."
    cargo build --target riscv32imac-unknown-none-elf
fi

echo "Testing with OpenSBI firmware..."

timeout 5s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -smp 1 \
    -m 128M \
    -nographic \
    -bios /usr/share/opensbi/generic/fw_jump.elf \
    -kernel "$KERNEL_PATH" || echo "QEMU finished (exit code: $?)"
