#!/bin/bash
# Unified build script for multi-architecture karatOS kernel

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
BUILD_MODE="debug"
ARCHITECTURE=""
CLEAN=false
VERBOSE=false

# Function to print colored output
print_colored() {
    echo -e "${1}${2}${NC}"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -a, --arch ARCH     Target architecture (arm|riscv)"
    echo "  -m, --mode MODE     Build mode (debug|release) [default: debug]"
    echo "  -c, --clean         Clean before building"
    echo "  -v, --verbose       Verbose output"
    echo "  -h, --help          Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --arch arm                 # Build for ARM in debug mode"
    echo "  $0 --arch riscv --mode release # Build for RISC-V in release mode"
    echo "  $0 --arch arm --clean         # Clean and build for ARM"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -a|--arch)
            ARCHITECTURE="$2"
            shift 2
            ;;
        -m|--mode)
            BUILD_MODE="$2"
            shift 2
            ;;
        -c|--clean)
            CLEAN=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Validate architecture
if [[ -z "$ARCHITECTURE" ]]; then
    print_colored $RED "Error: Architecture must be specified"
    show_usage
    exit 1
fi

if [[ "$ARCHITECTURE" != "arm" && "$ARCHITECTURE" != "riscv" ]]; then
    print_colored $RED "Error: Architecture must be 'arm' or 'riscv'"
    exit 1
fi

# Set target triple based on architecture
case $ARCHITECTURE in
    arm)
        TARGET="thumbv7em-none-eabi"
        FEATURES="arm"
        MEMORY_FILE="memory-arm.x"
        ;;
    riscv)
        TARGET="riscv32imac-unknown-none-elf"
        FEATURES="riscv"
        MEMORY_FILE="memory-riscv.x"
        ;;
esac

print_colored $BLUE "=== karatOS Multi-Architecture Build System ==="
print_colored $YELLOW "Architecture: $ARCHITECTURE"
print_colored $YELLOW "Target: $TARGET"
print_colored $YELLOW "Build Mode: $BUILD_MODE"
print_colored $YELLOW "Features: $FEATURES"
print_colored $YELLOW "Memory Layout: $MEMORY_FILE"

# Check if memory layout file exists
if [[ ! -f "kernel/$MEMORY_FILE" ]]; then
    print_colored $RED "Error: Memory layout file 'kernel/$MEMORY_FILE' not found"
    exit 1
fi

# Copy the appropriate memory layout file
cp "kernel/$MEMORY_FILE" kernel/memory.x
print_colored $GREEN "Using memory layout: kernel/$MEMORY_FILE"

# Install target if not already installed
if ! rustup target list --installed | grep -q "$TARGET"; then
    print_colored $YELLOW "Installing target: $TARGET"
    rustup target add "$TARGET"
fi

# Clean if requested
if [[ "$CLEAN" == true ]]; then
    print_colored $YELLOW "Cleaning previous builds..."
    cargo clean
fi

# Build command
BUILD_CMD="cargo build --manifest-path=kernel/Cargo.toml --target=$TARGET --features=$FEATURES"

if [[ "$BUILD_MODE" == "release" ]]; then
    BUILD_CMD="$BUILD_CMD --release"
fi

if [[ "$VERBOSE" == true ]]; then
    BUILD_CMD="$BUILD_CMD --verbose"
fi

print_colored $YELLOW "Building with command: $BUILD_CMD"

# Execute build
if $BUILD_CMD; then
    print_colored $GREEN "Build successful!"
    
    # Show binary information
    BUILD_DIR="kernel/target/$TARGET/$BUILD_MODE"
    BINARY="$BUILD_DIR/kernel"
    
    if [[ -f "$BINARY" ]]; then
        BINARY_SIZE=$(stat -c%s "$BINARY" 2>/dev/null || stat -f%z "$BINARY" 2>/dev/null)
        print_colored $GREEN "Binary: $BINARY"
        print_colored $GREEN "Size: $BINARY_SIZE bytes"
        
        # Show file command output if available
        if command -v file >/dev/null 2>&1; then
            print_colored $BLUE "File info: $(file "$BINARY")"
        fi
    fi
else
    print_colored $RED "Build failed!"
    exit 1
fi

# Clean up temporary memory.x
rm -f kernel/memory.x

print_colored $GREEN "Build complete!"
