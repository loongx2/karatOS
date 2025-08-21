#!/bin/bash

# Test kernel execution and memory contents
set -e

KERNEL_PATH="target/riscv32imac-unknown-none-elf/debug/kernel-riscv-minimal"

echo "=== Testing Kernel Execution with Memory Verification ==="

echo ""
echo "Step 1: Test file output"
rm -f uart_output.txt

timeout 3s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -m 128M \
    -bios none \
    -chardev file,id=char0,path=uart_output.txt \
    -serial chardev:char0 \
    -display none \
    -kernel "$KERNEL_PATH" || echo "Kernel execution finished"

echo ""
echo "UART output file:"
if [ -f uart_output.txt ]; then
    echo "Size: $(wc -c < uart_output.txt) bytes"
    if [ -s uart_output.txt ]; then
        echo "Contents:"
        cat uart_output.txt
        echo "--- End of file ---"
    else
        echo "File is empty"
    fi
else
    echo "No output file created"
fi

echo ""
echo "Step 2: Check memory contents with QEMU monitor"
echo "Running QEMU with monitor to check memory..."

# Create a script for QEMU monitor
cat > qemu_commands.txt << 'EOF'
x/4wx 0x80000100
x/4wx 0x80000108
info registers
quit
EOF

timeout 5s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -m 128M \
    -bios none \
    -serial null \
    -monitor stdio \
    -kernel "$KERNEL_PATH" < qemu_commands.txt || echo "Monitor session finished"

rm -f qemu_commands.txt

echo ""
echo "Test completed"
