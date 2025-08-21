#!/bin/bash

# Debug QEMU with monitor
set -e

KERNEL_PATH="target/riscv32imac-unknown-none-elf/debug/kernel-riscv-minimal"

echo "=== QEMU Debug Mode ==="

if [ ! -f "$KERNEL_PATH" ]; then
    echo "Error: Kernel not found. Building..."
    cargo build --target riscv32imac-unknown-none-elf
fi

echo "Starting QEMU in debug mode..."
echo "You can use QEMU monitor commands:"
echo "  info registers - show CPU registers"
echo "  info mtree - show memory tree"
echo "  x/10i \$pc - disassemble at PC"
echo "  quit - exit QEMU"

qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -smp 1 \
    -m 128M \
    -nographic \
    -bios none \
    -serial null \
    -monitor stdio \
    -kernel "$KERNEL_PATH"
