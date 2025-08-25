#!/bin/bash

# Multi-Architecture Build Script for RTOS
# Supports both ARM (Cortex-M) and RISC-V targets
# Now with configuration-driven build support

set -e

TARGET="${1:-all}"
BUILD_TYPE="${2:-debug}"
CONFIG_FILE="${3:-}"

echo "=== Multi-Architecture RTOS Build ==="
echo "Target: $TARGET"
echo "Build Type: $BUILD_TYPE"
if [ -n "$CONFIG_FILE" ]; then
    echo "Configuration: $CONFIG_FILE"
fi
echo ""

# Configuration-based builds
build_with_config() {
    local config_name="$1"
    echo "Building with configuration: $config_name"
    
    case "$config_name" in
        "arm_lm3s6965")
            echo "Configuration: ARM Cortex-M3 LM3S6965EVB"
            cd kernel
            if [ "$BUILD_TYPE" = "release" ]; then
                cargo build --target thumbv7m-none-eabi --features arm --release
            else
                cargo build --target thumbv7m-none-eabi --features arm
            fi
            echo "ARM LM3S6965 build complete: target/thumbv7m-none-eabi/$BUILD_TYPE/kernel"
            cd ..
            ;;
        "riscv_qemu")
            echo "Configuration: RISC-V RV32IMAC QEMU virt"
            cd kernel
            if [ "$BUILD_TYPE" = "release" ]; then
                cargo build --target riscv32imac-unknown-none-elf --features riscv --release
            else
                cargo build --target riscv32imac-unknown-none-elf --features riscv
            fi
            echo "RISC-V QEMU build complete: target/riscv32imac-unknown-none-elf/$BUILD_TYPE/kernel"
            cd ..
            ;;
        *)
            echo "Unknown configuration: $config_name"
            echo "Available configurations:"
            echo "  arm_lm3s6965  - ARM Cortex-M3 for LM3S6965EVB"
            echo "  riscv_qemu    - RISC-V RV32IMAC for QEMU virt"
            exit 1
            ;;
    esac
}

# Build commands
build_arm() {
    echo "Building ARM (Cortex-M) target..."
    cd kernel
    if [ "$BUILD_TYPE" = "release" ]; then
        cargo build --target thumbv7m-none-eabi --features arm --release
    else
        cargo build --target thumbv7m-none-eabi --features arm
    fi
    echo "ARM build complete: target/thumbv7m-none-eabi/$BUILD_TYPE/kernel"
    cd ..
}

build_riscv() {
    echo "Building RISC-V target..."
    cd kernel
    if [ "$BUILD_TYPE" = "release" ]; then
        cargo build --target riscv32imac-unknown-none-elf --features riscv --release
    else
        cargo build --target riscv32imac-unknown-none-elf --features riscv
    fi
    echo "RISC-V build complete: target/riscv32imac-unknown-none-elf/$BUILD_TYPE/kernel"
    cd ..
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
if [ -n "$CONFIG_FILE" ]; then
    # Configuration-driven build
    check_targets
    build_with_config "$CONFIG_FILE"
else
    # Traditional target-based build
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
            echo "Usage: $0 [arm|riscv|all] [debug|release] [config_name]"
            echo ""
            echo "Traditional builds:"
            echo "  $0 arm debug      # Build ARM debug version"
            echo "  $0 riscv release  # Build RISC-V release version"
            echo "  $0 all debug      # Build both targets debug"
            echo ""
            echo "Configuration-driven builds:"
            echo "  $0 config debug arm_lm3s6965   # Build ARM with LM3S6965 config"
            echo "  $0 config release riscv_qemu   # Build RISC-V with QEMU config"
            exit 1
            ;;
    esac
fi

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
