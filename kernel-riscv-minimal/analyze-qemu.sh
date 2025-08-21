#!/bin/bash

# Comprehensive QEMU test to understand the virt machine
set -e

KERNEL_PATH="target/riscv32imac-unknown-none-elf/debug/kernel-riscv-minimal"

echo "=== QEMU RISC-V virt Machine Analysis ==="

if [ ! -f "$KERNEL_PATH" ]; then
    echo "Building kernel..."
    cargo build --target riscv32imac-unknown-none-elf
fi

# First, let's see what the virt machine provides without our kernel
echo ""
echo "=== QEMU virt machine info (no kernel) ==="
timeout 2s qemu-system-riscv32 -machine virt -nographic -monitor stdio << 'EOF' || true
info mtree
info qtree
quit
EOF

echo ""
echo "=== Running our kernel for 3 seconds ==="
timeout 3s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -smp 1 \
    -m 128M \
    -nographic \
    -bios none \
    -kernel "$KERNEL_PATH" || echo "Kernel execution completed"

echo ""
echo "=== Test completed ==="
