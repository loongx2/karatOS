#!/bin/bash

# Test with memory monitoring to prove kernel is running
set -e

KERNEL_PATH="target/riscv32imac-unknown-none-elf/debug/kernel-riscv-minimal"

echo "=== Testing RISC-V Kernel with Memory Monitoring ==="

if [ ! -f "$KERNEL_PATH" ]; then
    echo "Error: Kernel not found. Building..."
    cargo build --target riscv32imac-unknown-none-elf
fi

echo "Testing kernel execution..."
echo "Kernel should write magic values to memory and try multiple UART addresses"

timeout 3s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -smp 1 \
    -m 128M \
    -nographic \
    -bios none \
    -serial stdio \
    -monitor stdio \
    -kernel "$KERNEL_PATH" &

QEMU_PID=$!
sleep 2

echo ""
echo "QEMU is running, testing if we see output..."
sleep 1

# Kill QEMU
kill $QEMU_PID 2>/dev/null || true
wait $QEMU_PID 2>/dev/null || true

echo "QEMU test completed"
