#!/bin/bash

# ARM QEMU Launch Script
# Run the ARM kernel in QEMU virt machine

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KERNEL_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
KERNEL_PATH="$(cd "$KERNEL_DIR/.." && pwd)/target/thumbv7m-none-eabi/debug/kernel"

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
    cargo build --target thumbv7m-none-eabi --features arm
    cd - > /dev/null
fi

log_info "Starting ARM QEMU..."
log_info "Kernel: $KERNEL_PATH"
log_info "Press Ctrl+A X to exit QEMU"

# Run QEMU with ARM Cortex-M3 configuration
qemu-system-arm \
    -machine lm3s6965evb \
    -cpu cortex-m3 \
    -semihosting \
    -serial mon:stdio \
    -kernel "$KERNEL_PATH" \
    "$@"
