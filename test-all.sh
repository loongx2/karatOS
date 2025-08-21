#!/bin/bash

# Multi-Architecture RTOS Test Suite
# Test both ARM and RISC-V kernels

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

test_arm() {
    log_info "Testing ARM kernel..."
    
    if [ ! -f "target/armv7a-none-eabi/debug/kernel" ]; then
        log_info "Building ARM kernel..."
        cargo build --target armv7a-none-eabi
    fi
    
    log_info "Running ARM QEMU test..."
    timeout 3s qemu-system-arm \
        -machine virt \
        -cpu cortex-a15 \
        -m 128M \
        -nographic \
        -bios none \
        -chardev file,id=char0,path=arm_test.txt \
        -serial chardev:char0 \
        -kernel target/armv7a-none-eabi/debug/kernel || log_info "ARM test completed"
    
    if [ -f arm_test.txt ] && [ -s arm_test.txt ]; then
        log_success "ARM kernel output detected:"
        head -5 arm_test.txt
    else
        log_warning "ARM: No output detected"
    fi
    
    rm -f arm_test.txt
}

test_riscv() {
    log_info "Testing RISC-V kernel..."
    
    cd kernel-riscv-minimal
    ./test.sh direct
    cd ..
}

test_both() {
    log_info "=== Multi-Architecture RTOS Test ==="
    echo ""
    
    test_arm
    echo ""
    test_riscv
    echo ""
    
    log_success "Multi-architecture testing completed"
}

show_status() {
    log_info "=== RTOS Status ==="
    echo ""
    
    echo "ARM Kernel:"
    if [ -f "target/armv7a-none-eabi/debug/kernel" ]; then
        echo "  ✓ Built: $(ls -lh target/armv7a-none-eabi/debug/kernel | awk '{print $5, $6, $7, $8}')"
    else
        echo "  ✗ Not built"
    fi
    
    echo ""
    echo "RISC-V Kernel:"
    if [ -f "kernel-riscv-minimal/target/riscv32imac-unknown-none-elf/debug/kernel-riscv-minimal" ]; then
        echo "  ✓ Built: $(ls -lh kernel-riscv-minimal/target/riscv32imac-unknown-none-elf/debug/kernel-riscv-minimal | awk '{print $5, $6, $7, $8}')"
        cd kernel-riscv-minimal
        echo "  ✓ Variant: $(./kernel.sh current | cut -d: -f2 | xargs)"
        cd ..
    else
        echo "  ✗ Not built"
    fi
}

build_all() {
    log_info "Building all architectures..."
    
    log_info "Building ARM kernel..."
    cargo build --target armv7a-none-eabi
    
    log_info "Building RISC-V kernel..."
    cd kernel-riscv-minimal
    ./build.sh
    cd ..
    
    log_success "All builds completed"
}

case "${1:-status}" in
    "status")
        show_status
        ;;
    "build")
        build_all
        ;;
    "test-arm")
        test_arm
        ;;
    "test-riscv")
        test_riscv
        ;;
    "test")
        test_both
        ;;
    "help"|"-h"|"--help")
        echo "Multi-Architecture RTOS Test Suite"
        echo ""
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  status       Show build status of both architectures"
        echo "  build        Build both ARM and RISC-V kernels"
        echo "  test-arm     Test ARM kernel only"
        echo "  test-riscv   Test RISC-V kernel only"
        echo "  test         Test both architectures"
        echo "  help         Show this help"
        echo ""
        echo "Architecture-specific commands:"
        echo "  For RISC-V: cd kernel-riscv-minimal && ./test.sh [command]"
        ;;
    *)
        log_error "Unknown command: $1"
        echo "Use '$0 help' for usage information"
        exit 1
        ;;
esac
