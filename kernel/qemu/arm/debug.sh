#!/bin/bash

# ARM QEMU Debug Script
# Launch ARM kernel with GDB debugging support

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KERNEL_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
KERNEL_PATH="$(cd "$KERNEL_DIR/.." && pwd)/target/thumbv7m-none-eabi/debug/kernel"
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
    log_info "Kernel not found, building ARM target..."
    cd "$KERNEL_DIR"
    cargo build --target thumbv7m-none-eabi --features arm
    cd - > /dev/null
fi

log_info "Starting ARM QEMU with GDB support..."
log_info "Kernel: $KERNEL_PATH"
log_info "GDB Port: $GDB_PORT"
log_warning "QEMU will wait for GDB connection..."
log_info "Connect with: arm-none-eabi-gdb $KERNEL_PATH"
log_info "Then in GDB: target remote localhost:$GDB_PORT"

# Run QEMU with GDB server
qemu-system-arm \
    -machine lm3s6965evb \
    -cpu cortex-m3 \
    -nographic \
    -semihosting-config enable=on,target=native \
    -gdb tcp::$GDB_PORT \
    -S \
    -kernel "$KERNEL_PATH" \
    "$@"
