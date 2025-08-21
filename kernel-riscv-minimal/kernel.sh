#!/bin/bash

# Kernel Variant Manager
# Switch between different kernel implementations for testing

set -e

CARGO_TOML="Cargo.toml"
SRC_DIR="src"

# Available kernel variants
declare -A KERNELS=(
    ["main"]="src/main.rs - Basic UART implementation"
    ["debug"]="src/main_debug.rs - Debug version with memory markers"
    ["blast"]="src/main_blast.rs - Simple UART blasting test"
    ["proper"]="src/main_proper_uart.rs - Properly initialized UART"
    ["test"]="src/main_test.rs - Multi-address UART testing"
    ["verbose"]="src/main_verbose.rs - Verbose output version"
    ["memory"]="src/main_memory_test.rs - Memory verification test"
)

show_current() {
    local current_path=$(grep 'path = "src/' "$CARGO_TOML" | sed 's/.*path = "src\/\(.*\)".*/\1/')
    echo "Current kernel: $current_path"
}

show_available() {
    echo "Available kernel variants:"
    for key in "${!KERNELS[@]}"; do
        echo "  $key: ${KERNELS[$key]}"
    done
}

switch_kernel() {
    local variant="$1"
    local file_name="main_${variant}.rs"
    
    if [ "$variant" = "main" ]; then
        file_name="main.rs"
    fi
    
    local src_path="src/$file_name"
    
    if [ ! -f "$src_path" ]; then
        echo "Error: Kernel variant '$variant' not found at $src_path"
        echo "Available variants:"
        show_available
        exit 1
    fi
    
    # Update Cargo.toml
    sed -i "s|path = \"src/.*\"|path = \"$src_path\"|" "$CARGO_TOML"
    
    echo "Switched to kernel variant: $variant"
    echo "File: $src_path"
    echo "Description: ${KERNELS[$variant]}"
    
    # Build the new variant
    echo "Building kernel..."
    cargo build --target riscv32imac-unknown-none-elf
}

case "${1:-current}" in
    "current")
        show_current
        ;;
    "list")
        show_available
        ;;
    "main"|"debug"|"blast"|"proper"|"test"|"verbose"|"memory")
        switch_kernel "$1"
        ;;
    *)
        echo "Kernel Variant Manager"
        echo ""
        echo "Usage: $0 [variant|command]"
        echo ""
        echo "Commands:"
        echo "  current      Show current kernel variant"
        echo "  list         List available variants"
        echo ""
        echo "Variants:"
        for key in "${!KERNELS[@]}"; do
            echo "  $key"
        done
        echo ""
        echo "Examples:"
        echo "  $0 debug     # Switch to debug kernel"
        echo "  $0 blast     # Switch to simple UART blast test"
        ;;
esac
