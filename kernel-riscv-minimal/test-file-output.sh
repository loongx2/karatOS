#!/bin/bash

# Test with explicit character device setup
set -e

KERNEL_PATH="target/riscv32imac-unknown-none-elf/debug/kernel-riscv-minimal"

echo "=== Testing QEMU with Character Device ==="

echo ""
echo "Test: Using chardev and file output"
rm -f uart_output.txt

timeout 5s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -m 128M \
    -bios none \
    -chardev file,id=char0,path=uart_output.txt \
    -serial chardev:char0 \
    -display none \
    -kernel "$KERNEL_PATH" || echo "QEMU finished"

echo ""
echo "Checking if anything was written to uart_output.txt:"
if [ -f uart_output.txt ]; then
    echo "File exists. Size: $(wc -c < uart_output.txt) bytes"
    echo "Contents:"
    cat uart_output.txt || echo "Error reading file"
    echo "--- End of file ---"
else
    echo "No output file created"
fi
