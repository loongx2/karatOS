#!/bin/bash

# RISC-V Kernel Test Suite
# Comprehensive testing for QEMU RISC-V virt machine

set -e

KERNEL_PATH="target/riscv32imac-unknown-none-elf/debug/kernel-riscv-minimal"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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

build_kernel() {
    log_info "Building RISC-V kernel..."
    if [ ! -f "$KERNEL_PATH" ]; then
        log_info "Kernel not found, building..."
        cargo build --target riscv32imac-unknown-none-elf
    fi
    
    if [ -f "$KERNEL_PATH" ]; then
        log_success "Kernel build completed"
        log_info "Entry point: $(readelf -h "$KERNEL_PATH" | grep Entry | awk '{print $4}')"
    else
        log_error "Kernel build failed"
        exit 1
    fi
}

test_direct_execution() {
    log_info "Testing direct kernel execution (no BIOS)..."
    rm -f direct_output.txt
    
    timeout 3s qemu-system-riscv32 \
        -machine virt \
        -cpu rv32 \
        -m 128M \
        -bios none \
        -chardev file,id=char0,path=direct_output.txt \
        -serial chardev:char0 \
        -display none \
        -kernel "$KERNEL_PATH" || log_info "QEMU execution completed"
    
    if [ -f direct_output.txt ] && [ -s direct_output.txt ]; then
        log_success "Direct execution produced output:"
        cat direct_output.txt
    else
        log_warning "Direct execution: No UART output detected"
    fi
}

test_opensbi_execution() {
    log_info "Testing with OpenSBI firmware..."
    rm -f opensbi_output.txt
    
    timeout 3s qemu-system-riscv32 \
        -machine virt \
        -cpu rv32 \
        -m 128M \
        -bios default \
        -chardev file,id=char0,path=opensbi_output.txt \
        -serial chardev:char0 \
        -display none \
        -kernel "$KERNEL_PATH" || log_info "OpenSBI execution completed"
    
    if [ -f opensbi_output.txt ] && [ -s opensbi_output.txt ]; then
        log_success "OpenSBI execution produced output:"
        cat opensbi_output.txt
    else
        log_warning "OpenSBI execution: No UART output detected"
    fi
}

test_interactive() {
    log_info "Starting interactive QEMU session..."
    log_info "Press Ctrl+A X to exit QEMU"
    
    qemu-system-riscv32 \
        -machine virt \
        -cpu rv32 \
        -m 128M \
        -bios none \
        -nographic \
        -kernel "$KERNEL_PATH"
}

debug_memory() {
    log_info "Debugging memory layout and registers..."
    
    cat > qemu_debug_commands.txt << 'EOF'
info mtree
info registers
x/10i 0x80000000
quit
EOF
    
    timeout 3s qemu-system-riscv32 \
        -machine virt \
        -cpu rv32 \
        -m 128M \
        -bios none \
        -serial null \
        -monitor stdio \
        -kernel "$KERNEL_PATH" < qemu_debug_commands.txt || log_info "Debug session completed"
    
    rm -f qemu_debug_commands.txt
}

show_device_tree() {
    log_info "Generating and displaying device tree..."
    
    # Generate device tree if not exists
    if [ ! -f virt.dtb ]; then
        timeout 2s qemu-system-riscv32 \
            -machine virt \
            -cpu rv32 \
            -m 128M \
            -machine dumpdtb=virt.dtb \
            -bios none \
            -kernel "$KERNEL_PATH" || log_info "Device tree generated"
    fi
    
    if [ -f virt.dtb ]; then
        log_success "Device tree information:"
        if command -v dtc &> /dev/null; then
            dtc -I dtb -O dts virt.dtb | grep -A 10 -B 5 serial || echo "UART info not found in device tree"
        else
            log_warning "dtc not installed. Install with: sudo apt install device-tree-compiler"
        fi
    fi
}

cleanup() {
    log_info "Cleaning up test artifacts..."
    rm -f direct_output.txt opensbi_output.txt qemu_debug_commands.txt
    log_success "Cleanup completed"
}

show_help() {
    echo "RISC-V Kernel Test Suite"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  build        Build the RISC-V kernel"
    echo "  test         Run all automated tests"
    echo "  direct       Test direct kernel execution"
    echo "  opensbi      Test with OpenSBI firmware"
    echo "  interactive  Start interactive QEMU session"
    echo "  debug        Debug memory and registers"
    echo "  dtb          Show device tree information"
    echo "  cleanup      Clean up test artifacts"
    echo "  help         Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 test      # Run all tests"
    echo "  $0 direct    # Test direct execution only"
    echo "  $0 debug     # Debug memory layout"
}

# Main execution
case "${1:-test}" in
    "build")
        build_kernel
        ;;
    "test")
        build_kernel
        test_direct_execution
        test_opensbi_execution
        show_device_tree
        ;;
    "direct")
        build_kernel
        test_direct_execution
        ;;
    "opensbi")
        build_kernel
        test_opensbi_execution
        ;;
    "interactive")
        build_kernel
        test_interactive
        ;;
    "debug")
        build_kernel
        debug_memory
        ;;
    "dtb")
        show_device_tree
        ;;
    "cleanup")
        cleanup
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
