#!/bin/bash

# RISC-V QEMU Launch Script for Development
# Usage: ./qemu-riscv32.sh [debug]

set -e

KERNEL_PATH="target/riscv32imac-unknown-none-elf/debug/kernel"
DEBUG_MODE="${1:-run}"

echo "=== RISC-V QEMU Launcher ==="
echo "Target: riscv32imac-unknown-none-elf"
echo "Machine: QEMU virt"
echo "Mode: $DEBUG_MODE"

# Check if kernel exists
if [ ! -f "$KERNEL_PATH" ]; then
    echo "Error: Kernel not found at $KERNEL_PATH"
    echo "Please build first with: cargo build --target riscv32imac-unknown-none-elf --features riscv"
    exit 1
fi

# Base QEMU command
QEMU_CMD="qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -smp 1 \
    -m 128M \
    -nographic \
    -serial stdio \
    -kernel $KERNEL_PATH"

if [ "$DEBUG_MODE" = "debug" ]; then
    echo "Starting QEMU in debug mode..."
    echo "Connect GDB with: riscv32-unknown-elf-gdb $KERNEL_PATH"
    echo "Then in GDB: target remote :1234"
    echo ""
    
    # Add debug options
    QEMU_CMD="$QEMU_CMD -s -S"
else
    echo "Starting QEMU in run mode..."
    echo "Press Ctrl+A, X to exit QEMU"
    echo ""
fi

echo "Command: $QEMU_CMD"
echo ""

# Execute QEMU
exec $QEMU_CMD
