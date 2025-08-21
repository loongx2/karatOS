#!/bin/bash

# RISC-V Device Tree Explorer
# Examine the QEMU virt machine device tree

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
    log_info "Building dummy kernel for device tree extraction..."
    cd "$KERNEL_DIR"
    cargo build --target riscv32imac-unknown-none-elf --features riscv
    cd - > /dev/null
fi

log_info "Extracting RISC-V virt machine device tree..."

# Generate device tree blob
timeout 2s qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -m 128M \
    -machine dumpdtb=riscv-virt.dtb \
    -bios none \
    -kernel "$KERNEL_PATH" || log_info "Device tree extracted"

if [ -f riscv-virt.dtb ]; then
    log_success "Device tree blob created: riscv-virt.dtb"
    
    if command -v dtc &> /dev/null; then
        log_info "Converting to readable format..."
        dtc -I dtb -O dts riscv-virt.dtb > riscv-virt.dts
        log_success "Device tree source created: riscv-virt.dts"
        
        log_info "UART configuration:"
        grep -A 10 -B 5 "serial@" riscv-virt.dts || echo "UART info not found"
        
        log_info "Memory configuration:"
        grep -A 5 -B 2 "memory@" riscv-virt.dts || echo "Memory info not found"
    else
        log_info "Install device-tree-compiler to view readable format:"
        log_info "  sudo apt install device-tree-compiler"
    fi
else
    log_info "Failed to extract device tree"
fi
