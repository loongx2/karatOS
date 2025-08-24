#!/bin/bash
# Comprehensive Test Script for karatOS Multi-Architecture Kernel
# Tests both ARM and RISC-V builds with various configurations

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

function test_arm() {
    log_info "🔧 Building ARM Cortex-M kernel..."
    
    if [ "$BUILD_TYPE" = "release" ]; then
        log_warning "Release profiles ignored for sub-packages - building in debug mode"
        cargo build --bin kernel-arm-working --target thumbv7m-none-eabi --features arm --quiet
    else
        cargo build --bin kernel-arm-working --target thumbv7m-none-eabi --features arm --quiet
    fi
    
    log_success "✅ ARM build successful"
    
    # Check binary size (debug builds only due to workspace limitation)
    local arm_binary="target/thumbv7m-none-eabi/debug/kernel-arm-working"
    if [ -f "$arm_binary" ]; then
        local arm_size=$(stat -c%s "$arm_binary" 2>/dev/null || stat -f%z "$arm_binary" 2>/dev/null || echo "unknown")
        log_info "ARM binary size: $arm_size bytes"
    fi
}

function test_riscv() {
    log_info "🔧 Building RISC-V kernel..."
    
    if [ "$BUILD_TYPE" = "release" ]; then
        log_warning "Release profiles ignored for sub-packages - building in debug mode"
        cargo build --bin kernel-riscv-simple --target riscv32imac-unknown-none-elf --features riscv --quiet
    else
        cargo build --bin kernel-riscv-simple --target riscv32imac-unknown-none-elf --features riscv --quiet
    fi
    
    log_success "✅ RISC-V build successful"
    
    # Check binary size (debug builds only due to workspace limitation)
    local riscv_binary="target/riscv32imac-unknown-none-elf/debug/kernel-riscv-simple"
    if [ -f "$riscv_binary" ]; then
        local riscv_size=$(stat -c%s "$riscv_binary" 2>/dev/null || stat -f%z "$riscv_binary" 2>/dev/null || echo "unknown")
        log_info "RISC-V binary size: $riscv_size bytes"
    fi
}

# Configuration
TEST_TYPE="${1:-all}"
BUILD_TYPE="${2:-debug}"

echo "🧪 karatOS Multi-Architecture Test Suite"
echo "========================================"
echo "Test Type: $TEST_TYPE"
echo "Build Type: $BUILD_TYPE"
echo ""

cd kernel

case "$TEST_TYPE" in
    "arm")
        log_info "Testing ARM architecture only..."
        test_arm
        ;;
    "riscv")
        log_info "Testing RISC-V architecture only..."
        test_riscv
        ;;
    "all"|*)
        log_info "Testing both ARM and RISC-V architectures..."
        test_arm
        test_riscv
        ;;
esac

echo ""
log_success "🎉 All tests completed successfully!"
