#!/bin/bash

# RISC-V QEMU Debug Script
# Launch RISC-V kernel with GDB debugging support

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KERNEL_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
KERNEL_PATH="$KERNEL_DIR/target/riscv32imac-unknown-none-elf/debug/kernel"
GDB_PORT="${GDB_PORT:-1234}"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if kernel exists
if [ ! -f "$KERNEL_PATH" ]; then
    log_info "Kernel not found, building RISC-V target..."
    cd "$KERNEL_DIR"
    cargo build --target riscv32imac-unknown-none-elf --features riscv
    cd - > /dev/null
fi

log_info "Starting RISC-V QEMU with GDB support..."
log_info "Kernel: $KERNEL_PATH"
log_info "GDB Port: $GDB_PORT"
log_warning "QEMU will wait for GDB connection..."
log_info "Connect with: riscv32-unknown-elf-gdb $KERNEL_PATH"
log_info "Then in GDB: target remote localhost:$GDB_PORT"

# Run QEMU with GDB server
qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -m 128M \
    -nographic \
    -bios none \
    -gdb tcp::$GDB_PORT \
    -S \
    -kernel "$KERNEL_PATH" \
    "$@"
