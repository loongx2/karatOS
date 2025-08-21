#!/bin/bash

# Test with QEMU debugging to see what's happening
set -e

KERNEL_PATH="target/riscv32imac-unknown-none-elf/debug/kernel-riscv-minimal"

echo "=== Testing RISC-V Kernel with Debug Info ==="

if [ ! -f "$KERNEL_PATH" ]; then
    echo "Error: Kernel not found. Building..."
    cargo build --target riscv32imac-unknown-none-elf
fi

echo "Kernel size and info:"
ls -la "$KERNEL_PATH"
file "$KERNEL_PATH"

echo ""
echo "Testing QEMU with debug output..."

# Test with QEMU machine info
echo "Available QEMU devices:"
timeout 3s qemu-system-riscv32 -machine virt -device help 2>/dev/null | head -10 || true

echo ""
echo "Testing kernel execution with tracing..."

# Run with execution tracing
timeout 5s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -smp 1 \
    -m 128M \
    -nographic \
    -bios none \
    -serial stdio \
    -kernel "$KERNEL_PATH" \
    -d guest_errors \
    -D qemu_debug.log || true

echo ""
echo "QEMU debug log (if any):"
if [ -f qemu_debug.log ]; then
    cat qemu_debug.log
    rm qemu_debug.log
else
    echo "No debug log created"
fi
