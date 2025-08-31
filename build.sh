#!/bin/bash
# karatOS Modular Build System v2.0
# Main entry point for the build system

# Get the directory where this script is located
BUILD_SYSTEM_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Load all modules
# Source modular components
source "build/modules/core.sh"
source "build/modules/arch.sh"
source "build/modules/qemu.sh"
source "build/modules/config.sh"

# Default values
TARGET="${TARGET:-all}"
BUILD_TYPE="${BUILD_TYPE:-debug}"
BOARD="${BOARD:-}"
TEST_MODE="${TEST_MODE:-false}"
INTERACTIVE_MODE="${INTERACTIVE_MODE:-false}"
CLEAN_MODE="${CLEAN_MODE:-false}"
VERBOSE="${VERBOSE:-false}"

# Parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            arm|riscv|all)
                TARGET="$1"
                shift
                ;;
            debug|release)
                BUILD_TYPE="$1"
                shift
                ;;
            --board|-b)
                BOARD="$2"
                shift 2
                ;;
            --test|-t)
                TEST_MODE=true
                shift
                ;;
            --interactive|-i)
                INTERACTIVE_MODE=true
                shift
                ;;
            --clean|-c)
                CLEAN_MODE=true
                shift
                ;;
            --verbose|-v)
                VERBOSE=true
                LOG_LEVEL="debug"
                shift
                ;;
            --timeout)
                QEMU_TIMEOUT="$2"
                shift 2
                ;;
            --interactive-timeout)
                QEMU_INTERACTIVE_TIMEOUT="$2"
                shift 2
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            --config)
                # Future: load specific config file
                shift 2
                ;;
            *)
                error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# Show help information
show_help() {
    cat << EOF
karatOS Modular Build System v2.0

USAGE:
    $0 [TARGET] [BUILD_TYPE] [OPTIONS]

TARGETS:
    arm         Build ARM Cortex-M target
    riscv       Build RISC-V target
    all         Build both targets

BUILD_TYPES:
    debug       Debug build (default)
    release     Release build

OPTIONS:
    -b, --board BOARD        Specify board configuration
    -t, --test               Run QEMU tests after build
    -i, --interactive        Run QEMU in interactive mode
    -c, --clean              Clean build artifacts first
    --timeout SECONDS        Set QEMU test timeout (default: 30s)
    --interactive-timeout SECONDS  Set QEMU interactive timeout (default: 300s)
    -v, --verbose            Enable verbose output
    -h, --help               Show this help

EXAMPLES:
    $0 arm                    # Build ARM debug
    $0 riscv release         # Build RISC-V release
    $0 all --test            # Build all and test
    $0 arm --board lm3s6965  # Build ARM for specific board
    $0 riscv --interactive   # Build and run RISC-V interactively
    $0 --clean all           # Clean and build all
    $0 all --test --timeout 60  # Build all with 60s test timeout
    $0 riscv --interactive --interactive-timeout 600  # 10min interactive session

CONFIGURATION:
    Configuration files are located in build/configs/
    - global.toml: Main configuration
    - Memory templates in build/templates/

EOF
}

# Main build function
build_target() {
    local target="$1"
    local build_type="$2"
    local board="$3"

    log_info "Building $target target ($build_type)"

    # Validate architecture
    validate_architecture "$target"

    # Load target configuration
    load_target_config "$target"

    # Load board configuration if specified
    if [[ -n "$board" ]]; then
        load_board_config "$target" "$board"
    fi

    # Setup build environment
    setup_build_environment "$target" "$board"

    # Generate memory layout
    generate_memory_layout "$target" "$board"

    # Execute cargo build
    execute_cargo_build "$target" "$build_type"

    # Validate build output
    validate_build_output "$target" "$build_type"

    # Save build configuration
    save_build_config "$target" "$build_type" "$board"

    log_success "$target build completed successfully"
}

# Execute cargo build
execute_cargo_build() {
    local target="$1"
    local build_type="$2"

    cd "$KERNEL_DIR"

    # Get cargo arguments
    local cargo_args
    cargo_args=$(get_cargo_args "$target" "$build_type")

    log_info "Running: cargo build $cargo_args"

    if cargo build $cargo_args; then
        log_success "Cargo build completed"
    else
        error "Cargo build failed"
    fi
}

# Main build orchestration
execute_build() {
    if [[ "$CLEAN_MODE" == true ]]; then
        log_info "Cleaning build artifacts"
        cd "$KERNEL_DIR"
        cargo clean
        log_success "Clean completed"
    fi

    case "$TARGET" in
        arm|riscv)
            build_target "$TARGET" "$BUILD_TYPE" "$BOARD"

            if [[ "$TEST_MODE" == true || "$INTERACTIVE_MODE" == true ]]; then
                if [[ "$INTERACTIVE_MODE" == true ]]; then
                    run_qemu_interactive "$TARGET" "$BOARD" "$BUILD_TYPE"
                else
                    run_qemu_test "$TARGET" "$BOARD" "$BUILD_TYPE"
                fi
            fi
            ;;
        all)
            log_info "Building all targets"

            # Build ARM
            build_target "arm" "$BUILD_TYPE" "$BOARD"

            # Build RISC-V
            build_target "riscv" "$BUILD_TYPE" ""

            if [[ "$TEST_MODE" == true || "$INTERACTIVE_MODE" == true ]]; then
                if [[ "$INTERACTIVE_MODE" == true ]]; then
                    echo "Interactive mode not supported for 'all' target"
                    echo "Use specific target with --interactive"
                else
                    # Test both targets
                    run_qemu_test "arm" "$BOARD" "$BUILD_TYPE"
                    run_qemu_test "riscv" "" "$BUILD_TYPE"
                fi
            fi
            ;;
        *)
            error "Invalid target: $TARGET"
            ;;
    esac
}

# Show build summary
show_summary() {
    echo ""
    echo "=== Build Summary ==="

    if [[ "$TARGET" == "all" || "$TARGET" == "arm" ]]; then
        local arm_triple
        arm_triple=$(get_target_triple "arm")
        echo "ARM binary: target/$arm_triple/$BUILD_TYPE/kernel"
        echo "  Run with: ./qemu-arm.sh"
    fi

    if [[ "$TARGET" == "all" || "$TARGET" == "riscv" ]]; then
        local riscv_triple
        riscv_triple=$(get_target_triple "riscv")
        echo "RISC-V binary: target/$riscv_triple/$BUILD_TYPE/kernel"
        echo "  Run with: ./qemu-riscv.sh"
    fi

    if [[ "$TEST_MODE" == true ]]; then
        echo ""
        echo "Tests completed successfully"
    fi

    echo ""
    log_success "Build system execution completed"
}

# Main function
main() {
    # Initialize configuration
    init_config

    # Parse command line arguments
    parse_arguments "$@"

    # Validate configuration
    validate_config "$TARGET" "$BUILD_TYPE" "$BOARD"

    # Show configuration if verbose
    if [[ "$VERBOSE" == true ]]; then
        show_config
    fi

    log_info "Starting karatOS build system"
    log_info "Target: $TARGET, Build Type: $BUILD_TYPE, Board: ${BOARD:-default}"

    # Execute build
    execute_build

    # Show summary
    show_summary
}

# Run main function with all arguments
main "$@"
