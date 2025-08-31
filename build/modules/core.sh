#!/bin/bash
# karatOS Build System - Core Module
# Core utility functions and build orchestration

# Global variables
declare -r BUILD_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
declare -r KERNEL_DIR="$BUILD_ROOT/kernel"
declare -r CONFIG_DIR="$BUILD_ROOT/build/configs"
declare -r TEMPLATE_DIR="$BUILD_ROOT/build/templates"
declare -r LOG_FILE="$BUILD_ROOT/build.log"

# Color codes for output
declare -r RED='\033[0;31m'
declare -r GREEN='\033[0;32m'
declare -r YELLOW='\033[1;33m'
declare -r BLUE='\033[0;34m'
declare -r NC='\033[0m' # No Color

# Logging functions
log_debug() {
    [[ "${LOG_LEVEL:-info}" == "debug" ]] && echo -e "${BLUE}[DEBUG]${NC} $*" | tee -a "$LOG_FILE"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2 | tee -a "$LOG_FILE"
}

# Error handling
error() {
    log_error "$*"
    exit 1
}

warning() {
    log_warning "$*"
}

# Validation functions
validate_target() {
    local target="$1"
    case "$target" in
        arm|riscv|all) return 0 ;;
        *) error "Invalid target: $target. Must be 'arm', 'riscv', or 'all'" ;;
    esac
}

validate_build_type() {
    local build_type="$1"
    case "$build_type" in
        debug|release) return 0 ;;
        *) error "Invalid build type: $build_type. Must be 'debug' or 'release'" ;;
    esac
}

# Configuration loading
load_toml_value() {
    local file="$1"
    local key="$2"
    local default="${3:-}"

    if [[ ! -f "$file" ]]; then
        echo "$default"
        return
    fi

    # Parse TOML table structure
    # Handle keys like "targets.arm.triple"
    local section=""
    local property=""

    if [[ "$key" == *"."* ]]; then
        section=$(echo "$key" | cut -d'.' -f1-2)
        property=$(echo "$key" | cut -d'.' -f3-)
    else
        property="$key"
    fi

    # Find the section and extract the property value
    if [[ -n "$section" ]]; then
        # Look for [section] and then the property within that section
        awk -v section="[$section]" -v property="$property" '
            $0 == section { in_section=1; next }
            /^\[/ && in_section { in_section=0 }
            in_section && $1 == property && $2 == "=" {
                # Handle different value formats
                value=$0
                sub(/.*= /, "", value)
                
                # Remove quotes
                gsub(/^"/, "", value)
                gsub(/"$/, "", value)
                
                # Handle arrays: ["item1", "item2"] -> item1,item2
                if (match(value, /^\[.*\]$/)) {
                    # Remove brackets and quotes, join with commas
                    sub(/^\[/, "", value)
                    sub(/\]$/, "", value)
                    gsub(/"/, "", value)
                    gsub(/, /, ",", value)
                }
                
                # Remove trailing commas
                gsub(/,$/, "", value)
                
                print value
                exit
            }
        ' "$file" || echo "$default"
    else
        # Simple key=value at root level
        grep "^$property = " "$file" 2>/dev/null | sed 's/.*= "\(.*\)"/\1/' | sed 's/.*= \(.*\)/\1/' | tr -d '"' || echo "$default"
    fi
}

# Get target triple from configuration
get_target_triple() {
    local target="$1"
    load_toml_value "$CONFIG_DIR/global.toml" "targets.$target.triple"
}

# Get target features from configuration
get_target_features() {
    local target="$1"
    load_toml_value "$CONFIG_DIR/global.toml" "targets.$target.features"
}

# Get memory template for target
get_memory_template() {
    local target="$1"
    load_toml_value "$CONFIG_DIR/global.toml" "targets.$target.memory_template"
}

# Get QEMU configuration
get_qemu_config() {
    local target="$1"
    local board="$2"
    local key

    # Determine the correct QEMU config key
    if [[ -n "$board" ]]; then
        key="qemu.${target}_${board}"
    else
        # Default board mapping
        case "$target" in
            arm) key="qemu.arm_lm3s6965" ;;
            riscv) key="qemu.riscv_qemu" ;;
            *) key="qemu.${target}_qemu" ;;
        esac
    fi

    # Get command
    local cmd
    cmd=$(load_toml_value "$CONFIG_DIR/global.toml" "$key.command")

    # Get args (simplified - would need better TOML parsing for arrays)
    local args
    case "$target-$board" in
        "arm-lm3s6965"|"arm-")
            args="-M lm3s6965evb -nographic -semihosting-config enable=on,target=native -serial mon:stdio"
            ;;
        "riscv-qemu"|"riscv-")
            args="-machine virt -cpu rv32 -smp 1 -m 128M -nographic -bios none -serial mon:stdio"
            ;;
        *)
            args="-nographic"
            ;;
    esac

    echo "$cmd|$args"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Setup logging
setup_logging() {
    # Create log file
    touch "$LOG_FILE"

    # Add timestamp header
    echo "=== karatOS Build Log - $(date) ===" >> "$LOG_FILE"
    echo "Command: $0 $*" >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"
}

# Cleanup function
cleanup() {
    local exit_code=$?
    if [[ $exit_code -ne 0 ]]; then
        log_error "Build failed with exit code $exit_code"
    fi
    log_info "Build completed at $(date)"
}

# Set trap for cleanup
trap cleanup EXIT
