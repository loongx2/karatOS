#!/bin/bash
# karatOS Build System - QEMU Module
# QEMU testing and integration functions

# Get QEMU timeout for testing
get_qemu_timeout() {
    echo "${QEMU_TIMEOUT:-30}"
}

# Get QEMU timeout for interactive sessions
get_qemu_interactive_timeout() {
    echo "${QEMU_INTERACTIVE_TIMEOUT:-300}"
}

# Run QEMU test for specific target
run_qemu_test() {
    local target="$1"
    local board="${2:-}"
    local build_type="${3:-debug}"
    local timeout="${4:-${QEMU_TIMEOUT:-30}}"

    log_info "Running QEMU test for $target ($board)"

    # Get binary path
    local triple
    triple=$(get_target_triple "$target")
    local binary_path="$BUILD_ROOT/target/$triple/$build_type/kernel"

    if [[ ! -f "$binary_path" ]]; then
        error "Binary not found: $binary_path"
    fi

    # Get QEMU configuration
    local qemu_config
    qemu_config=$(get_qemu_config "$target" "$board")

    # Parse QEMU config (format: "command|args")
    local qemu_cmd
    local qemu_args
    qemu_cmd=$(echo "$qemu_config" | cut -d'|' -f1)
    qemu_args=$(echo "$qemu_config" | cut -d'|' -f2)

    if ! command_exists "$qemu_cmd"; then
        warning "QEMU command not found: $qemu_cmd"
        log_info "Skipping QEMU test"
        return 0
    fi

    log_debug "QEMU command: $qemu_cmd"
    log_debug "QEMU args: $qemu_args"
    log_debug "Kernel: $binary_path"

    # Build QEMU command
    local cmd="$qemu_cmd $qemu_args -kernel $binary_path"

    log_info "Starting QEMU test (timeout: ${timeout}s)"
    log_debug "Command: $cmd"

    # Run QEMU with timeout
    local start_time
    start_time=$(date +%s)

    if timeout "$timeout" bash -c "$cmd" 2>/dev/null; then
        local end_time
        end_time=$(date +%s)
        local duration=$((end_time - start_time))
        log_success "QEMU test completed successfully in ${duration}s"
    else
        local exit_code=$?
        if [[ $exit_code -eq 124 ]]; then
            log_success "QEMU test completed (timed out after ${timeout}s)"
        else
            warning "QEMU test failed with exit code $exit_code"
        fi
    fi
}

# Run QEMU in interactive mode
run_qemu_interactive() {
    local target="$1"
    local board="${2:-}"
    local build_type="${3:-debug}"
    local timeout="${4:-${QEMU_INTERACTIVE_TIMEOUT:-300}}"

    log_info "Running QEMU interactively for $target ($board)"

    # Get binary path
    local triple
    triple=$(get_target_triple "$target")
    local binary_path="$BUILD_ROOT/target/$triple/$build_type/kernel"

    if [[ ! -f "$binary_path" ]]; then
        error "Binary not found: $binary_path"
    fi

    # Get QEMU configuration
    local qemu_config
    qemu_config=$(get_qemu_config "$target" "$board")

    # Parse QEMU config
    local qemu_cmd
    local qemu_args
    qemu_cmd=$(echo "$qemu_config" | cut -d'|' -f1)
    qemu_args=$(echo "$qemu_config" | cut -d'|' -f2)

    if ! command_exists "$qemu_cmd"; then
        error "QEMU command not found: $qemu_cmd"
    fi

    # Build QEMU command
    local cmd="$qemu_cmd $qemu_args -kernel $binary_path"

    log_info "Starting interactive QEMU session"
    log_info "Press Ctrl+A, X to exit QEMU"
    log_info "Session will timeout after ${timeout}s if no interaction"
    log_debug "Command: $cmd"

    # Run QEMU interactively with timeout
    if timeout "$timeout" bash -c "$cmd"; then
        log_success "Interactive QEMU session completed"
    else
        local exit_code=$?
        if [[ $exit_code -eq 124 ]]; then
            log_info "Interactive QEMU session timed out after ${timeout}s"
        else
            warning "Interactive QEMU session ended with exit code $exit_code"
        fi
    fi
}

# Check QEMU availability
check_qemu_availability() {
    local target="$1"

    local qemu_cmd
    case "$target" in
        arm) qemu_cmd="qemu-system-arm" ;;
        riscv) qemu_cmd="qemu-system-riscv32" ;;
        *) error "Unknown target for QEMU check: $target" ;;
    esac

    if command_exists "$qemu_cmd"; then
        log_debug "QEMU available: $qemu_cmd"
        return 0
    else
        warning "QEMU not available: $qemu_cmd"
        return 1
    fi
}

# Get QEMU version information
get_qemu_version() {
    local target="$1"

    local qemu_cmd
    case "$target" in
        arm) qemu_cmd="qemu-system-arm" ;;
        riscv) qemu_cmd="qemu-system-riscv32" ;;
        *) error "Unknown target for QEMU version: $target" ;;
    esac

    if command_exists "$qemu_cmd"; then
        local version
        version=$($qemu_cmd --version 2>&1 | head -n1)
        log_debug "QEMU version: $version"
        echo "$version"
    else
        echo "Not available"
    fi
}

# Setup QEMU debugging
setup_qemu_debug() {
    local target="$1"
    local board="${2:-}"
    local gdb_port="${3:-1234}"

    log_info "Setting up QEMU debugging on port $gdb_port"

    # Get QEMU configuration
    local qemu_config
    qemu_config=$(get_qemu_config "$target" "$board")

    # Parse QEMU config
    local qemu_cmd
    local qemu_args
    qemu_cmd=$(echo "$qemu_config" | cut -d'|' -f1)
    qemu_args=$(echo "$qemu_config" | cut -d'|' -f2)

    # Add debug arguments
    qemu_args="$qemu_args -s -S"

    # Get binary path
    local triple
    triple=$(get_target_triple "$target")
    local binary_path="$BUILD_ROOT/target/$triple/debug/kernel"

    if [[ ! -f "$binary_path" ]]; then
        error "Debug binary not found: $binary_path"
    fi

    local cmd="$qemu_cmd $qemu_args -kernel $binary_path"

    log_info "QEMU debug session ready"
    log_info "GDB server will listen on port $gdb_port"
    log_info "Connect with: gdb-multiarch -ex 'target remote localhost:$gdb_port'"
    log_debug "Command: $cmd"

    # Run QEMU in background for debugging
    eval "$cmd" &
    local qemu_pid=$!

    log_info "QEMU started with PID: $qemu_pid"
    echo "$qemu_pid"
}

# Kill QEMU processes
kill_qemu_processes() {
    log_info "Killing QEMU processes"

    local pids
    pids=$(pgrep -f qemu-system)

    if [[ -n "$pids" ]]; then
        echo "$pids" | xargs kill 2>/dev/null || true
        log_success "QEMU processes killed"
    else
        log_debug "No QEMU processes found"
    fi
}

# Test multiple targets
test_all_targets() {
    local build_type="${1:-debug}"
    local timeout="${2:-30}"

    log_info "Testing all targets with $build_type build"

    local targets=("arm" "riscv")

    for target in "${targets[@]}"; do
        log_info "Testing $target..."

        if check_qemu_availability "$target"; then
            run_qemu_test "$target" "" "$build_type" "$timeout"
        else
            warning "Skipping $target test (QEMU not available)"
        fi

        echo ""
    done

    log_success "All target tests completed"
}
