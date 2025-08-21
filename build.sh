#!/bin/bash

# Multi-Architecture Build Script for RTOS
# Supports both ARM (Cortex-M) and RISC-V targets

set -e

TARGET="${1:-all}"
BUILD_TYPE="${2:-debug}"

echo "=== Multi-Architecture RTOS Build ==="
echo "Target: $TARGET"
echo "Build Type: $BUILD_TYPE"
echo ""

# Build commands
build_arm() {
    echo "Building ARM (Cortex-M) target..."
    if [ "$BUILD_TYPE" = "release" ]; then
        cargo build -p kernel --target thumbv7m-none-eabi --features arm --release
    else
        cargo build -p kernel --target thumbv7m-none-eabi --features arm
    fi
    echo "ARM build complete: target/thumbv7m-none-eabi/$BUILD_TYPE/kernel"
}

build_riscv() {
    echo "Building RISC-V target..."
    if [ "$BUILD_TYPE" = "release" ]; then
        cargo build -p kernel --target riscv32imac-unknown-none-elf --features riscv --release
    else
        cargo build -p kernel --target riscv32imac-unknown-none-elf --features riscv
    fi
    echo "RISC-V build complete: target/riscv32imac-unknown-none-elf/$BUILD_TYPE/kernel"
}

# Check if targets are installed
check_targets() {
    echo "Checking Rust targets..."
    
    if ! rustup target list --installed | grep -q thumbv7m-none-eabi; then
        echo "Installing ARM target..."
        rustup target add thumbv7m-none-eabi
    fi
    
    if ! rustup target list --installed | grep -q riscv32imac-unknown-none-elf; then
        echo "Installing RISC-V target..."
        rustup target add riscv32imac-unknown-none-elf
    fi
    
    echo "Targets ready."
    echo ""
}

# Main build logic
case "$TARGET" in
    "arm")
        check_targets
        build_arm
        ;;
    "riscv")
        check_targets
        build_riscv
        ;;
    "all")
        check_targets
        build_arm
        echo ""
        build_riscv
        ;;
    *)
        echo "Usage: $0 [arm|riscv|all] [debug|release]"
        echo ""
        echo "Examples:"
        echo "  $0 arm debug      # Build ARM debug version"
        echo "  $0 riscv release  # Build RISC-V release version"
        echo "  $0 all debug      # Build both targets debug"
        exit 1
        ;;
esac

echo ""
echo "=== Build Summary ==="
if [ "$TARGET" = "all" ] || [ "$TARGET" = "arm" ]; then
    echo "ARM binary: target/thumbv7m-none-eabi/$BUILD_TYPE/kernel"
    echo "  Run with: ./qemu-arm.sh"
    echo "  Debug with: ./debug-interactive.sh"
fi

if [ "$TARGET" = "all" ] || [ "$TARGET" = "riscv" ]; then
    echo "RISC-V binary: target/riscv32imac-unknown-none-elf/$BUILD_TYPE/kernel"
    echo "  Run with: ./qemu-riscv32.sh"
    echo "  Debug with: ./debug-riscv.sh"
fi

echo ""
echo "Build completed successfully!"
