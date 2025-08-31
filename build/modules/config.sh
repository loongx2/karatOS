#!/bin/bash
# karatOS Build System - Configuration Module
# Configuration loading and management

# Load global configuration
load_global_config() {
    local config_file="$CONFIG_DIR/global.toml"

    if [[ ! -f "$config_file" ]]; then
        warning "Global config file not found: $config_file"
        return 1
    fi

    log_debug "Loading global configuration from $config_file"

    # Load basic settings
    LOG_LEVEL=$(load_toml_value "$config_file" "build.log_level" "info")
    PARALLEL_JOBS=$(load_toml_value "$config_file" "build.parallel_jobs" "1")
    VERBOSE=$(load_toml_value "$config_file" "build.verbose" "false")
    QEMU_TIMEOUT=$(load_toml_value "$config_file" "build.qemu_timeout" "30")
    QEMU_INTERACTIVE_TIMEOUT=$(load_toml_value "$config_file" "build.qemu_interactive_timeout" "300")

    log_debug "Configuration loaded:"
    log_debug "  LOG_LEVEL=$LOG_LEVEL"
    log_debug "  PARALLEL_JOBS=$PARALLEL_JOBS"
    log_debug "  VERBOSE=$VERBOSE"
    log_debug "  QEMU_TIMEOUT=$QEMU_TIMEOUT"
    log_debug "  QEMU_INTERACTIVE_TIMEOUT=$QEMU_INTERACTIVE_TIMEOUT"

    return 0
}

# Load target-specific configuration
load_target_config() {
    local target="$1"
    local config_file="$CONFIG_DIR/global.toml"

    if [[ ! -f "$config_file" ]]; then
        error "Target config file not found: $config_file"
    fi

    log_debug "Loading target configuration for $target"

    # Load target-specific settings
    TARGET_TRIPLE=$(load_toml_value "$config_file" "targets.$target.triple")
    TARGET_FEATURES=$(load_toml_value "$config_file" "targets.$target.features")
    MEMORY_TEMPLATE=$(load_toml_value "$config_file" "targets.$target.memory_template")

    if [[ -z "$TARGET_TRIPLE" ]]; then
        error "Target triple not found for $target"
    fi

    log_debug "Target configuration:"
    log_debug "  TRIPLE=$TARGET_TRIPLE"
    log_debug "  FEATURES=$TARGET_FEATURES"
    log_debug "  MEMORY_TEMPLATE=$MEMORY_TEMPLATE"
}

# Load board-specific configuration
load_board_config() {
    local target="$1"
    local board="$2"
    local config_file="$CONFIG_DIR/global.toml"

    if [[ ! -f "$config_file" ]]; then
        warning "Board config file not found: $config_file"
        return 1
    fi

    log_debug "Loading board configuration for $target/$board"

    # Load board-specific settings
    BOARD_NAME=$(load_toml_value "$config_file" "boards.${target}_${board}.name" "$board")
    QEMU_MACHINE=$(load_toml_value "$config_file" "boards.${target}_${board}.qemu_machine")

    log_debug "Board configuration:"
    log_debug "  NAME=$BOARD_NAME"
    log_debug "  QEMU_MACHINE=$QEMU_MACHINE"
}

# Save build configuration
save_build_config() {
    local target="$1"
    local build_type="$2"
    local board="${3:-}"

    local config_file="$BUILD_ROOT/.build_cache"

    cat > "$config_file" << EOF
# karatOS Build Cache - $(date)
TARGET=$target
BUILD_TYPE=$build_type
BOARD=$board
TIMESTAMP=$(date +%s)
EOF

    log_debug "Build configuration saved to $config_file"
}

# Load cached build configuration
load_build_cache() {
    local config_file="$BUILD_ROOT/.build_cache"

    if [[ ! -f "$config_file" ]]; then
        return 1
    fi

    # Source the cache file
    source "$config_file"

    log_debug "Build cache loaded from $config_file"
    return 0
}

# Validate configuration
validate_config() {
    local target="$1"
    local build_type="$2"
    local board="${3:-}"

    log_info "Validating configuration..."

    # Validate target
    validate_target "$target"

    # Validate build type
    validate_build_type "$build_type"

    # Check if target triple is available (skip for 'all')
    if [[ "$target" != "all" ]]; then
        local triple
        triple=$(get_target_triple "$target")
        if [[ -z "$triple" ]]; then
            error "No triple configured for target: $target"
        fi
    fi

    # Check if memory template exists
    local template
    template=$(get_memory_template "$target")
    if [[ -n "$template" && ! -f "$TEMPLATE_DIR/$template" ]]; then
        error "Memory template not found: $TEMPLATE_DIR/$template"
    fi

    log_success "Configuration validation passed"
}

# Show current configuration
show_config() {
    echo "=== karatOS Build Configuration ==="
    echo "Build Root: $BUILD_ROOT"
    echo "Kernel Dir: $KERNEL_DIR"
    echo "Config Dir: $CONFIG_DIR"
    echo "Template Dir: $TEMPLATE_DIR"
    echo "Log File: $LOG_FILE"
    echo ""
    echo "Global Settings:"
    echo "  Log Level: ${LOG_LEVEL:-info}"
    echo "  Parallel Jobs: ${PARALLEL_JOBS:-1}"
    echo "  Verbose: ${VERBOSE:-false}"
    echo ""

    if [[ -n "${TARGET:-}" ]]; then
        echo "Current Target: $TARGET"
        echo "  Triple: ${TARGET_TRIPLE:-unknown}"
        echo "  Features: ${TARGET_FEATURES:-none}"
        echo "  Memory Template: ${MEMORY_TEMPLATE:-none}"
        echo ""
    fi
}

# Initialize configuration
init_config() {
    log_debug "Initializing build configuration"

    # Load global config
    if ! load_global_config; then
        warning "Using default configuration"
    fi

    # Setup logging
    setup_logging

    log_debug "Configuration initialized"
}
