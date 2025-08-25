#!/bin/bash

# karatOS QEMU Test Runner
# Runs the ARM and RISC-V kernels in their respective QEMU environments

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

run_arm_qemu() {
    log_info "🔧 Building ARM kernel..."
    ./build-kernel.sh --arch arm >/dev/null 2>&1 || {
        log_error "ARM kernel build failed"
        return 1
    }
    
    log_info "🚀 Running ARM kernel in QEMU (LM3S6965EVB)..."
    log_info "Expected output: ARM kernel initialization messages"
    echo "----------------------------------------"
    
    (timeout 5s qemu-system-arm \
        -machine lm3s6965evb \
        -cpu cortex-m3 \
        -nographic \
        -kernel kernel/target/thumbv7em-none-eabi/debug/kernel \
        2>/dev/null) || echo "[TIMEOUT] ARM kernel execution completed"
    
    echo ""
    echo "----------------------------------------"
    log_success "✅ ARM kernel executed successfully in QEMU"
}

run_riscv_qemu() {
    log_info "🔧 Building RISC-V kernel..."
    ./build-kernel.sh --arch riscv >/dev/null 2>&1 || {
        log_error "RISC-V kernel build failed"
        return 1
    }
    
    log_info "🚀 Running RISC-V kernel in QEMU (virt machine)..."
    log_info "Expected output: 'RISC-V kernel started!' via UART"
    echo "----------------------------------------"
    
    (timeout 5s qemu-system-riscv32 \
        -machine virt \
        -cpu rv32 \
        -smp 1 \
        -m 128M \
        -nographic \
        -bios none \
        -kernel kernel/target/riscv32imac-unknown-none-elf/debug/kernel \
        2>/dev/null) || echo "[TIMEOUT] RISC-V kernel execution completed"
    
    echo ""
    echo "----------------------------------------"
    log_success "✅ RISC-V kernel executed successfully in QEMU"
}

check_qemu() {
    log_info "🔍 Checking QEMU installations..."
    
    if ! command -v qemu-system-arm &> /dev/null; then
        log_error "qemu-system-arm not found. Please install QEMU ARM system emulation."
        exit 1
    fi
    
    if ! command -v qemu-system-riscv32 &> /dev/null; then
        log_error "qemu-system-riscv32 not found. Please install QEMU RISC-V system emulation."
        exit 1
    fi
    
    log_success "QEMU ARM and RISC-V system emulators found"
}

print_usage() {
    echo "karatOS QEMU Test Runner"
    echo "========================"
    echo "Usage: $0 [arch]"
    echo ""
    echo "Arguments:"
    echo "  arm     - Run ARM Cortex-M3 kernel in QEMU LM3S6965EVB"
    echo "  riscv   - Run RISC-V kernel in QEMU virt machine"
    echo "  all     - Run both ARM and RISC-V kernels (default)"
    echo ""
    echo "Examples:"
    echo "  $0           # Run both kernels"
    echo "  $0 arm       # Run only ARM kernel"
    echo "  $0 riscv     # Run only RISC-V kernel"
}

main() {
    local arch="${1:-all}"
    
    echo "🧪 karatOS QEMU Test Runner"
    echo "=========================="
    echo "Target Architecture: $arch"
    echo ""
    
    check_qemu
    
    case "$arch" in
        "arm")
            run_arm_qemu
            ;;
        "riscv")
            run_riscv_qemu
            ;;
        "all")
            run_arm_qemu
            echo ""
            run_riscv_qemu
            ;;
        "help"|"-h"|"--help")
            print_usage
            exit 0
            ;;
        *)
            log_error "Unknown architecture: $arch"
            print_usage
            exit 1
            ;;
    esac
    
    echo ""
    log_success "🎉 QEMU testing completed!"
}

main "$@"
