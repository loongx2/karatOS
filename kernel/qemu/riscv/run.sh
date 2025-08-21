#!/bin/bash

# RISC-V QEMU Launch Script
# Run the RISC-V kernel in QEMU virt machine

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KERNEL_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
KERNEL_PATH="$KERNEL_DIR/target/riscv32imac-unknown-none-elf/debug/kernel"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Check if kernel exists
if [ ! -f "$KERNEL_PATH" ]; then
    log_info "Kernel not found, building RISC-V target..."
    cd "$KERNEL_DIR"
    cargo build --target riscv32imac-unknown-none-elf --features riscv
    cd - > /dev/null
fi

log_info "Starting RISC-V QEMU..."
log_info "Kernel: $KERNEL_PATH"
log_info "Press Ctrl+A X to exit QEMU"

# Run QEMU with RISC-V configuration
qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -m 128M \
    -nographic \
    -bios none \
    -kernel "$KERNEL_PATH" \
    "$@"
