#!/bin/bash

# Comprehensive OpenSBI test
set -e

KERNEL_PATH="target/riscv32imac-unknown-none-elf/debug/kernel-riscv-minimal"

echo "=== OpenSBI + Kernel Test ==="

echo ""
echo "Test 1: OpenSBI with file output"
rm -f opensbi_output.txt

timeout 5s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -m 128M \
    -bios default \
    -chardev file,id=char0,path=opensbi_output.txt \
    -serial chardev:char0 \
    -display none \
    -kernel "$KERNEL_PATH" || echo "OpenSBI test finished"

echo ""
if [ -f opensbi_output.txt ]; then
    echo "OpenSBI output (size: $(wc -c < opensbi_output.txt) bytes):"
    if [ -s opensbi_output.txt ]; then
        cat opensbi_output.txt
        echo "--- End of output ---"
    else
        echo "Output file is empty"
    fi
else
    echo "No output file created"
fi

echo ""
echo "Test 2: Direct kernel without bios"
rm -f direct_output.txt

timeout 3s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -m 128M \
    -bios none \
    -chardev file,id=char0,path=direct_output.txt \
    -serial chardev:char0 \
    -display none \
    -kernel "$KERNEL_PATH" || echo "Direct test finished"

if [ -f direct_output.txt ]; then
    echo "Direct kernel output (size: $(wc -c < direct_output.txt) bytes):"
    if [ -s direct_output.txt ]; then
        cat direct_output.txt
        echo "--- End of output ---"
    else
        echo "Output file is empty"
    fi
else
    echo "No output file created"
fi

echo ""
echo "Test 3: Check if we can disassemble the kernel at entry point"
timeout 2s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -m 128M \
    -bios none \
    -serial null \
    -monitor stdio \
    -kernel "$KERNEL_PATH" << 'EOF' || echo "Disassembly test finished"
x/10i 0x80000000
quit
EOF

echo ""
echo "All tests completed"
