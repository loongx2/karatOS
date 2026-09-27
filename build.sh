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
source "build/modules/size.sh"

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
            arm|arm-v8m|riscv|riscv-imc|riscv64|all)
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
    arm         Build ARM Cortex-M3/M4/M7 target (thumbv7m)
    arm-v8m     Build ARM v8-M target (Cortex-M33/M55, thumbv8m.main)
    riscv       Build RISC-V RV32IMAC target
    riscv-imc   Build RISC-V RV32IMC target (no atomics, e.g. ESP32-C3 class)
    riscv64     Build RISC-V RV64GC target (e.g. SiFive U54)
    all         Build primary ARM + RISC-V targets

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
    $0 riscv-imc release     # Build RV32IMC variant (no A extension)
    $0 riscv64 release       # Build RV64GC variant (SiFive U54-class)
    $0 arm-v8m --board mps3-an547 --test  # Cortex-M55/M33 on MPS3-AN547
    $0 arm-v8m --board musca-b1           # Cortex-M33 on Musca-B1

CONFIGURATION:
    Configuration files are located in build/configs/
    - global.toml: Main configuration
    - Memory templates in build/templates/

EOF
}

# Main build function
# ---------------------------------------------------------------------------
# Phase 3: loadable module image build (modules/hello -> raw .bin)
# Builds the module for the SAME target triple, extracts the raw image at
# its slot address, patches size+CRC (ci/elf_to_bin.py), and exports
# KARATOS_MODULE_BIN for the kernel's build script to embed.
# ---------------------------------------------------------------------------
build_module_image() {
    local target="$1"
    unset KARATOS_MODULE_BIN

    command -v python3 >/dev/null 2>&1 || { warning "python3 not found - skipping module image"; return 0; }

    local triple
    triple=$(get_target_triple "$target") || return 0
    local build_type="release"

    # Per-target KAPI/slot geography (must match karatos-kapi + linker
    # templates so the module binds to the kernel export table).
    case "$target" in
        arm)     export KARATOS_KAPI_ADDR=0x0001F000; export KARATOS_MODULE_SLOT=0x20004000 ;;
        arm-v8m) export KARATOS_KAPI_ADDR=0x1001F000; export KARATOS_MODULE_SLOT=0x30004000 ;;
        *)       export KARATOS_KAPI_ADDR=0x80010000; export KARATOS_MODULE_SLOT=0x80008000 ;;
    esac

    log_info "Building loadable module image (hello) for $triple"
    if ! (cd "$BUILD_SYSTEM_DIR/modules/hello" && cargo build --release --target "$triple" --bin hello 2>>"$BUILD_SYSTEM_DIR/build.log"); then
        warning "module build failed - kernel will build WITHOUT embedded module"
        return 0
    fi

    local elf="$BUILD_SYSTEM_DIR/target/$triple/$build_type/hello"
    local bin="$BUILD_SYSTEM_DIR/modules/hello/hello-$triple.bin"
    if python3 "$BUILD_SYSTEM_DIR/ci/elf_to_bin.py" "$elf" "$bin" >>"$BUILD_SYSTEM_DIR/build.log" 2>&1; then
        export KARATOS_MODULE_BIN="$bin"
        log_success "Module image ready: $bin"
    else
        warning "module post-processing failed - kernel will build WITHOUT embedded module"
        return 0
    fi

    # Phase 4: pack the module image(s) into the flash STORE image and export
    # its base address for the QEMU loader device (models bootloader provisioning).
    local board_id
    case "$target" in
        arm)      board_id=1 ;;
        arm-v8m)  board_id=5 ;;
        riscv-imc) board_id=3 ;;
        riscv64)  board_id=4 ;;
        *)        board_id=2 ;;
    esac
    local store="$BUILD_SYSTEM_DIR/store-$triple.bin"
    if python3 "$BUILD_SYSTEM_DIR/ci/build_store.py" --board-id "$board_id" -o "$store" "$bin" >>"$BUILD_SYSTEM_DIR/build.log" 2>&1; then
        export KARATOS_STORE_BIN="$store"
        case "$target" in
            arm)     export KARATOS_STORE_BASE=0x00022000 ;;
            arm-v8m) export KARATOS_STORE_BASE=0x10021000 ;;
          *) export KARATOS_STORE_BASE=0x83000000 ;;
        esac
        log_success "Flash store ready: $store @ $KARATOS_STORE_BASE"
    else
        warning "store build failed - kernel will run without provisioned modules"
    fi
    return 0
}

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

    # Phase 3: build the loadable module image FIRST so the kernel can embed
    # it (kernel/build.rs picks up KARATOS_MODULE_BIN; on any failure the
    # kernel still builds without the module demo).
    build_module_image "$target" || true

    # Execute cargo build
    execute_cargo_build "$target" "$build_type"

    # Validate build output
    validate_build_output "$target" "$build_type"

    # Enforce the 64 kB ROM/SRAM footprint gate (Milestone 3).
    # Debug builds are informational only; release builds hard-fail on breach.
    report_footprint "$target" "$build_type"
    if [[ "$build_type" == "release" ]]; then
        check_size_budget "$target" "$build_type" || error "Footprint gate failed for $target"
    fi

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
        arm|arm-v8m|riscv|riscv-imc|riscv64)
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

            # Build ARM (Cortex-M3 baseline)
            build_target "arm" "$BUILD_TYPE" "$BOARD"

            # Build RISC-V (RV32IMAC)
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
