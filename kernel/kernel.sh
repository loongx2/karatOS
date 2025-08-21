#!/bin/bash

# Multi-Architecture Kernel Build and Test Script
# Unified interface for building and testing both ARM and RISC-V kernels

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

build_arm() {
    log_info "Building ARM kernel..."
    cd "$SCRIPT_DIR"
    cargo build --target armv7a-none-eabi --features arm
    
    if [ -f "../target/armv7a-none-eabi/debug/kernel" ]; then
        log_success "ARM kernel built successfully"
        ls -lh ../target/armv7a-none-eabi/debug/kernel
    else
        log_error "ARM kernel build failed"
        return 1
    fi
}

build_riscv() {
    log_info "Building RISC-V kernel..."
    cd "$SCRIPT_DIR"
    cargo build --target riscv32imac-unknown-none-elf --features riscv
    
    if [ -f "../target/riscv32imac-unknown-none-elf/debug/kernel" ]; then
        log_success "RISC-V kernel built successfully"
        ls -lh ../target/riscv32imac-unknown-none-elf/debug/kernel
    else
        log_error "RISC-V kernel build failed"
        return 1
    fi
}

build_all() {
    log_info "Building all architectures..."
    build_arm
    build_riscv
    log_success "All builds completed"
}

clean() {
    log_info "Cleaning build artifacts..."
    cd "$SCRIPT_DIR"
    cargo clean
    log_success "Clean completed"
}

test_arm() {
    log_info "Testing ARM kernel..."
    build_arm
    "$SCRIPT_DIR/qemu/arm/run.sh" &
    local qemu_pid=$!
    
    # Let it run for a few seconds then kill
    sleep 3
    kill $qemu_pid 2>/dev/null || true
    wait $qemu_pid 2>/dev/null || true
    log_success "ARM test completed"
}

test_riscv() {
    log_info "Testing RISC-V kernel..."
    build_riscv
    timeout 3s "$SCRIPT_DIR/qemu/riscv/run.sh" || log_info "RISC-V test completed"
}

test_all() {
    log_info "Testing all architectures..."
    test_arm
    test_riscv
    log_success "All tests completed"
}

run_arm() {
    log_info "Running ARM kernel interactively..."
    build_arm
    "$SCRIPT_DIR/qemu/arm/run.sh"
}

run_riscv() {
    log_info "Running RISC-V kernel interactively..."
    build_riscv
    "$SCRIPT_DIR/qemu/riscv/run.sh"
}

debug_arm() {
    log_info "Starting ARM kernel in debug mode..."
    build_arm
    "$SCRIPT_DIR/qemu/arm/debug.sh"
}

debug_riscv() {
    log_info "Starting RISC-V kernel in debug mode..."
    build_riscv
    "$SCRIPT_DIR/qemu/riscv/debug.sh"
}

show_status() {
    log_info "=== Kernel Build Status ==="
    echo ""
    
    echo "ARM Kernel:"
    if [ -f "$SCRIPT_DIR/target/armv7a-none-eabi/debug/kernel" ]; then
        echo "  ✓ Built: $(ls -lh $SCRIPT_DIR/target/armv7a-none-eabi/debug/kernel | awk '{print $5, $6, $7, $8}')"
    else
        echo "  ✗ Not built"
    fi
    
    echo ""
    echo "RISC-V Kernel:"
    if [ -f "$SCRIPT_DIR/target/riscv32imac-unknown-none-elf/debug/kernel" ]; then
        echo "  ✓ Built: $(ls -lh $SCRIPT_DIR/target/riscv32imac-unknown-none-elf/debug/kernel | awk '{print $5, $6, $7, $8}')"
    else
        echo "  ✗ Not built"
    fi
    
    echo ""
    echo "Device Configuration:"
    echo "  ARM:    PL011 UART @ 0x09000000, 128MB @ 0x40000000"
    echo "  RISC-V: NS16550A UART @ 0x10000000, 128MB @ 0x80000000"
}

show_help() {
    echo "Multi-Architecture Kernel Build System"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Build Commands:"
    echo "  build-arm       Build ARM kernel only"
    echo "  build-riscv     Build RISC-V kernel only"
    echo "  build           Build both architectures"
    echo "  clean           Clean all build artifacts"
    echo ""
    echo "Test Commands:"
    echo "  test-arm        Test ARM kernel (quick)"
    echo "  test-riscv      Test RISC-V kernel (quick)"
    echo "  test            Test both architectures"
    echo ""
    echo "Run Commands:"
    echo "  run-arm         Run ARM kernel interactively"
    echo "  run-riscv       Run RISC-V kernel interactively"
    echo ""
    echo "Debug Commands:"
    echo "  debug-arm       Start ARM kernel with GDB"
    echo "  debug-riscv     Start RISC-V kernel with GDB"
    echo ""
    echo "Info Commands:"
    echo "  status          Show build status"
    echo "  help            Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 build        # Build both architectures"
    echo "  $0 test-arm     # Quick ARM test"
    echo "  $0 run-riscv    # Interactive RISC-V session"
}

# Main command handling
case "${1:-status}" in
    "build-arm")
        build_arm
        ;;
    "build-riscv")
        build_riscv
        ;;
    "build")
        build_all
        ;;
    "clean")
        clean
        ;;
    "test-arm")
        test_arm
        ;;
    "test-riscv")
        test_riscv
        ;;
    "test")
        test_all
        ;;
    "run-arm")
        run_arm
        ;;
    "run-riscv")
        run_riscv
        ;;
    "debug-arm")
        debug_arm
        ;;
    "debug-riscv")
        debug_riscv
        ;;
    "status")
        show_status
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    *)
        log_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac
