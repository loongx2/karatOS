#!/bin/bash

# ARM QEMU Launch Script
# Run the ARM kernel in QEMU virt machine

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KERNEL_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
KERNEL_PATH="$(cd "$KERNEL_DIR/.." && pwd)/target/armv7a-none-eabi/debug/kernel"

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
    log_info "Kernel not found, building ARM target..."
    cd "$KERNEL_DIR"
    cargo build --target armv7a-none-eabi --features arm
    cd - > /dev/null
fi

log_info "Starting ARM QEMU..."
log_info "Kernel: $KERNEL_PATH"
log_info "Press Ctrl+A X to exit QEMU"

# Run QEMU with ARM configuration
qemu-system-arm \
    -machine virt \
    -cpu cortex-a15 \
    -m 128M \
    -nographic \
    -semihosting-config enable=on,target=native \
    -kernel "$KERNEL_PATH" \
    "$@"
