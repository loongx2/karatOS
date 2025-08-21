#!/bin/bash

# RISC-V Kernel Build Script
# Clean build and verification

set -e

TARGET="riscv32imac-unknown-none-elf"
KERNEL_NAME="kernel-riscv-minimal"

echo "=== RISC-V Kernel Build ==="

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Build kernel
echo "Building kernel for target: $TARGET"
cargo build --target "$TARGET"

# Verify build
KERNEL_PATH="target/$TARGET/debug/$KERNEL_NAME"

if [ -f "$KERNEL_PATH" ]; then
    echo "✓ Build successful"
    echo "✓ Kernel: $KERNEL_PATH"
    
    # Show binary info
    echo ""
    echo "Binary information:"
    file "$KERNEL_PATH"
    echo ""
    readelf -h "$KERNEL_PATH" | grep -E "(Entry point|Machine)"
    echo ""
    ls -lh "$KERNEL_PATH"
else
    echo "✗ Build failed - kernel not found"
    exit 1
fi

echo ""
echo "Build completed successfully!"
echo "Use './test.sh' to run tests"
