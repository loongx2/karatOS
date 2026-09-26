#!/bin/bash
# karatOS Build System - Architecture Module
# Architecture-specific build logic and configuration

# Setup build environment for specific architecture
setup_build_environment() {
    local target="$1"
    local board="${2:-}"

    log_info "Setting up build environment for $target"

    case "$target" in
        arm|arm-v8m)
            setup_arm_environment "$board" "$target"
            ;;
        riscv|riscv-imc|riscv64)
            setup_riscv_environment "$board" "$target"
            ;;
        *)
            error "Unsupported architecture: $target"
            ;;
    esac
}

# ARM-specific setup
# Usage: setup_arm_environment <board> [variant]
#   variant: arm (Cortex-M3/M4/M7, thumbv7m) | arm-v8m (Cortex-M33/M55, thumbv8m.main)
setup_arm_environment() {
    local board="$1"
    local variant="${2:-arm}"

    log_debug "Setting up ARM environment for board: $board (variant: $variant)"

    # Board-specific configuration
    case "$board" in
        mps3-an547|an547)
            export ARM_BOARD="mps3-an547"
            export ARM_QEMU_MACHINE="mps3-an547"
            # MPS3-AN547 hosts Cortex-M55 (latest) / M33 images - v8-M mainline
            variant="arm-v8m"
            ;;
        musca-b1)
            export ARM_BOARD="musca-b1"
            export ARM_QEMU_MACHINE="musca-b1"
            variant="arm-v8m"
            ;;
        lm3s6965|*)
            export ARM_BOARD="lm3s6965evb"
            export ARM_QEMU_MACHINE="lm3s6965evb"
            ;;
    esac

    # Variant-specific toolchain triple
    case "$variant" in
        arm-v8m)
            export ARM_TARGET="thumbv8m.main-none-eabi"
            ;;
        *)
            export ARM_TARGET="thumbv7m-none-eabi"
            ;;
    esac

    export ARM_FEATURES="arm"
    export ARM_VARIANT="$variant"

    log_debug "ARM_TARGET=$ARM_TARGET"
    log_debug "ARM_FEATURES=$ARM_FEATURES"
    log_debug "ARM_BOARD=$ARM_BOARD"
}

# RISC-V-specific setup
# Usage: setup_riscv_environment <board> [variant]
#   variant: riscv (RV32IMAC) | riscv-imc (RV32IMC, e.g. ESP32-C3 class) | riscv64 (RV64GC)
setup_riscv_environment() {
    local board="$1"
    local variant="${2:-riscv}"

    log_debug "Setting up RISC-V environment for board: $board (variant: $variant)"

    # Variant-specific toolchain triple
    case "$variant" in
        riscv-imc)
            export RISCV_TARGET="riscv32imc-unknown-none-elf"
            ;;
        riscv64)
            export RISCV_TARGET="riscv64gc-unknown-none-elf"
            ;;
        *)
            export RISCV_TARGET="riscv32imac-unknown-none-elf"
            ;;
    esac

    # Board-specific configuration
    case "$board" in
        qemu|*)
            export RISCV_BOARD="virt"
            export RISCV_QEMU_MACHINE="virt"
            ;;
    esac

    export RISCV_FEATURES="riscv"
    export RISCV_VARIANT="$variant"

    log_debug "RISCV_TARGET=$RISCV_TARGET"
    log_debug "RISCV_FEATURES=$RISCV_FEATURES"
    log_debug "RISCV_BOARD=$RISCV_BOARD"
}

# Generate memory layout from template
generate_memory_layout() {
    local target="$1"
    local board="${2:-}"

    log_info "Generating memory layout for $target"

    local template_file="$TEMPLATE_DIR/$(get_memory_template "$target")"
    local output_file="$KERNEL_DIR/memory.x"

    if [[ ! -f "$template_file" ]]; then
        error "Memory template not found: $template_file"
    fi

    log_debug "Using template: $template_file"

    # For now, just copy the template
    # In a full implementation, this would substitute variables
    cp "$template_file" "$output_file"

    log_success "Memory layout generated: $output_file"
}

# Get build target triple
get_build_target() {
    local target="$1"
    echo "${!target^^}_TARGET"
}

# Get build features
get_build_features() {
    local target="$1"
    echo "${!target^^}_FEATURES"
}

# Validate architecture support
validate_architecture() {
    local target="$1"

    # Check if target is supported
    case "$target" in
        arm|arm-v8m|riscv|riscv-imc|riscv64)
            log_debug "Architecture $target is supported"
            ;;
        *)
            error "Unsupported architecture: $target"
            ;;
    esac

    # Check if Rust target is installed
    local triple
    triple=$(get_target_triple "$target")

    if ! rustup target list --installed | grep -q "$triple"; then
        log_info "Installing Rust target: $triple"
        rustup target add "$triple" || error "Failed to install target $triple"
    fi

    log_success "Architecture $target validated"
}

# Get architecture-specific Cargo arguments
get_cargo_args() {
    local target="$1"
    local build_type="$2"

    local args=()

    # Add target
    local triple
    triple=$(get_target_triple "$target")
    args+=("--target" "$triple")

    # Add features
    local features
    features=$(get_target_features "$target")
    if [[ -n "$features" ]]; then
        args+=("--features" "$features")
    fi

    # Add build type
    if [[ "$build_type" == "release" ]]; then
        args+=("--release")
    fi

    # Return space-separated string
    echo "${args[*]}"
}

# Architecture-specific post-build validation
validate_build_output() {
    local target="$1"
    local build_type="$2"

    local triple
    triple=$(get_target_triple "$target")

    local binary_path="$BUILD_ROOT/target/$triple/$build_type/kernel"

    if [[ ! -f "$binary_path" ]]; then
        error "Build failed: binary not found at $binary_path"
    fi

    local size
    size=$(stat -c%s "$binary_path" 2>/dev/null || stat -f%z "$binary_path" 2>/dev/null || echo "unknown")

    log_success "Build validation passed"
    log_info "Binary: $binary_path"
    log_info "Size: $size bytes"
}
