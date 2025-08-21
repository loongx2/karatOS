#!/bin/bash

# Test different QEMU serial configurations
set -e

KERNEL_PATH="target/riscv32imac-unknown-none-elf/debug/kernel-riscv-minimal"

echo "=== Testing Different QEMU Serial Configurations ==="

echo ""
echo "Test 1: Default configuration"
timeout 3s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -m 128M \
    -nographic \
    -bios none \
    -kernel "$KERNEL_PATH" || echo "Test 1 completed"

echo ""
echo "Test 2: With explicit serial stdio"
timeout 3s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -m 128M \
    -bios none \
    -serial stdio \
    -display none \
    -kernel "$KERNEL_PATH" || echo "Test 2 completed"

echo ""
echo "Test 3: With chardev"
timeout 3s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -m 128M \
    -bios none \
    -chardev stdio,id=char0 \
    -serial chardev:char0 \
    -display none \
    -kernel "$KERNEL_PATH" || echo "Test 3 completed"

echo ""
echo "Test 4: Check if QEMU can show device tree"
echo "Let's see what devices are available:"
timeout 2s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -m 128M \
    -machine dumpdtb=virt.dtb \
    -bios none \
    -kernel "$KERNEL_PATH" || echo "DTB dumped"

if [ -f virt.dtb ]; then
    echo "Device tree blob created. Let's examine it:"
    which dtc && dtc -I dtb -O dts virt.dtb | grep -A 10 -B 5 uart || echo "dtc not available"
fi

echo "All tests completed"
