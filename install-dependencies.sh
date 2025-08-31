#!/bin/bash
# karatOS Dependency Installation Script
# Supports multiple Linux distributions and macOS

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect OS and distribution
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v lsb_release >/dev/null 2>&1; then
            DISTRO=$(lsb_release -si | tr '[:upper:]' '[:lower:]')
            VERSION=$(lsb_release -sr)
        elif [[ -f /etc/os-release ]]; then
            . /etc/os-release
            DISTRO=$ID
            VERSION=$VERSION_ID
        elif [[ -f /etc/debian_version ]]; then
            DISTRO="debian"
        elif [[ -f /etc/redhat-release ]]; then
            DISTRO="rhel"
        else
            DISTRO="unknown"
        fi
        OS="linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
        DISTRO="macos"
    else
        OS="unknown"
        DISTRO="unknown"
    fi

    log_info "Detected OS: $OS ($DISTRO)"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install Rust toolchain (Optimized)
install_rust() {
    log_info "Checking Rust installation..."

    if command_exists rustc && command_exists cargo; then
        log_info "Rust is already installed"
        return 0
    fi

    log_info "Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

    # Source cargo environment
    source "$HOME/.cargo/env"

    # Verify installation
    if command_exists rustc && command_exists cargo; then
        log_info "Rust installed successfully"
        rustc --version
        cargo --version
    else
        log_error "Failed to install Rust"
        return 1
    fi
}

# Install Rust targets (Optimized)
install_rust_targets() {
    log_info "Checking Rust targets..."

    local targets=("thumbv7em-none-eabihf" "thumbv7em-none-eabi" "riscv32imac-unknown-none-elf" "riscv32imc-unknown-none-elf")
    local needs_install=false

    for target in "${targets[@]}"; do
        if ! rustup target list --installed | grep -q "^$target"; then
            needs_install=true
            break
        fi
    done

    if [[ "$needs_install" == "true" ]]; then
        log_info "Installing missing Rust targets..."
        for target in "${targets[@]}"; do
            if ! rustup target list --installed | grep -q "^$target"; then
                log_info "Installing target: $target"
                rustup target add "$target"
            fi
        done
    else
        log_info "All required Rust targets already installed"
    fi
}

# Install QEMU for Linux (Optimized)
install_qemu_linux() {
    case $DISTRO in
        ubuntu|debian|linuxmint|pop)
            log_info "Installing QEMU on Ubuntu/Debian..."

            # Check if already installed to avoid unnecessary updates
            if ! command_exists qemu-system-arm || ! command_exists qemu-system-riscv32; then
                log_info "Updating package list (this may take a moment)..."
                sudo apt-get update
                log_info "Installing QEMU packages..."
                sudo apt-get install -y qemu-system-arm qemu-system-riscv32 qemu-system-misc
            else
                log_info "QEMU already installed, skipping package installation"
            fi
            ;;
        fedora|rhel|centos|rocky|almalinux)
            log_info "Installing QEMU on Fedora/RHEL..."

            if ! command_exists qemu-system-arm || ! command_exists qemu-system-riscv32; then
                sudo dnf install -y qemu-system-arm qemu-system-riscv32 qemu-system-misc
            else
                log_info "QEMU already installed, skipping package installation"
            fi
            ;;
        arch|manjaro|endeavouros)
            log_info "Installing QEMU on Arch Linux..."

            if ! command_exists qemu-system-arm || ! command_exists qemu-system-riscv32; then
                sudo pacman -S --noconfirm qemu-system-arm qemu-system-riscv32
            else
                log_info "QEMU already installed, skipping package installation"
            fi
            ;;
        opensuse*|sles)
            log_info "Installing QEMU on openSUSE..."

            if ! command_exists qemu-system-arm || ! command_exists qemu-system-riscv32; then
                sudo zypper install -y qemu-arm qemu-riscv32
            else
                log_info "QEMU already installed, skipping package installation"
            fi
            ;;
        *)
            log_warning "Unknown Linux distribution: $DISTRO"
            log_warning "Please install QEMU manually:"
            echo "  Ubuntu/Debian: sudo apt-get install qemu-system-arm qemu-system-riscv32"
            echo "  Fedora/RHEL: sudo dnf install qemu-system-arm qemu-system-riscv32"
            echo "  Arch: sudo pacman -S qemu-system-arm qemu-system-riscv32"
            return 1
            ;;
    esac
}

# Install QEMU for macOS
install_qemu_macos() {
    if command_exists brew; then
        log_info "Installing QEMU on macOS with Homebrew..."
        brew install qemu
    else
        log_warning "Homebrew not found. Please install Homebrew first:"
        echo "  /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        echo "Then run: brew install qemu"
        return 1
    fi
}

# Install QEMU
install_qemu() {
    if command_exists qemu-system-arm && command_exists qemu-system-riscv32; then
        log_info "QEMU is already installed"
        qemu-system-arm --version | head -1
        qemu-system-riscv32 --version | head -1
    else
        log_info "Installing QEMU..."
        if [[ "$OS" == "linux" ]]; then
            install_qemu_linux
        elif [[ "$OS" == "macos" ]]; then
            install_qemu_macos
        else
            log_error "Unsupported OS for automatic QEMU installation"
            return 1
        fi

        if command_exists qemu-system-arm && command_exists qemu-system-riscv32; then
            log_success "QEMU installed successfully"
            qemu-system-arm --version | head -1
            qemu-system-riscv32 --version | head -1
        else
            log_error "QEMU installation failed"
            return 1
        fi
    fi
}

# Install optional debugging tools
install_debug_tools() {
    log_info "Installing optional debugging tools..."

    if [[ "$OS" == "linux" ]]; then
        case $DISTRO in
            ubuntu|debian|linuxmint|pop)
                sudo apt-get install -y gdb-multiarch openocd
                ;;
            fedora|rhel|centos|rocky|almalinux)
                sudo dnf install -y gdb openocd
                ;;
            arch|manjaro|endeavouros)
                sudo pacman -S --noconfirm gdb openocd
                ;;
            opensuse*|sles)
                sudo zypper install -y gdb openocd
                ;;
            *)
                log_warning "Unknown distribution for debug tools installation"
                return 1
                ;;
        esac
    elif [[ "$OS" == "macos" ]]; then
        if command_exists brew; then
            brew install gdb openocd
        else
            log_warning "Homebrew not found for debug tools installation"
            return 1
        fi
    fi

    log_success "Debug tools installed (optional)"
}

# Install build essentials (Optimized)
install_build_tools() {
    log_info "Checking build tools..."

    # Check if basic tools are already installed
    local needs_install=false

    if ! command_exists gcc && ! command_exists clang; then
        needs_install=true
    fi

    if ! command_exists make; then
        needs_install=true
    fi

    if ! command_exists git; then
        needs_install=true
    fi

    if ! command_exists curl; then
        needs_install=true
    fi

    if [[ "$needs_install" == "true" ]]; then
        log_info "Installing missing build tools..."

        if [[ "$OS" == "linux" ]]; then
            case $DISTRO in
                ubuntu|debian|linuxmint|pop)
                    log_info "Installing build tools for Ubuntu/Debian..."
                    sudo apt-get install -y build-essential git curl
                    ;;
                fedora|rhel|centos|rocky|almalinux)
                    log_info "Installing build tools for Fedora/RHEL..."
                    sudo dnf install -y gcc gcc-c++ make git curl
                    ;;
                arch|manjaro|endeavouros)
                    log_info "Installing build tools for Arch Linux..."
                    sudo pacman -S --noconfirm base-devel git curl
                    ;;
                opensuse*|sles)
                    log_info "Installing build tools for openSUSE..."
                    sudo zypper install -y gcc gcc-c++ make git curl
                    ;;
                *)
                    log_warning "Unknown distribution for build tools installation"
                    ;;
            esac
        elif [[ "$OS" == "macos" ]]; then
            if command_exists brew; then
                log_info "Installing build tools for macOS..."
                brew install git curl
            fi
        fi
    else
        log_info "Build tools already installed"
    fi
}

# Install debugging tools (Optimized)
install_debug_tools() {
    log_info "Checking debugging tools..."

    local needs_install=false

    if ! command_exists gdb; then
        needs_install=true
    fi

    if ! command_exists openocd; then
        needs_install=true
    fi

    if [[ "$needs_install" == "true" ]]; then
        log_info "Installing missing debugging tools..."

        if [[ "$OS" == "linux" ]]; then
            case $DISTRO in
                ubuntu|debian|linuxmint|pop)
                    log_info "Installing debug tools for Ubuntu/Debian..."
                    sudo apt-get install -y gdb openocd
                    ;;
                fedora|rhel|centos|rocky|almalinux)
                    log_info "Installing debug tools for Fedora/RHEL..."
                    sudo dnf install -y gdb openocd
                    ;;
                arch|manjaro|endeavouros)
                    log_info "Installing debug tools for Arch Linux..."
                    sudo pacman -S --noconfirm gdb openocd
                    ;;
                opensuse*|sles)
                    log_info "Installing debug tools for openSUSE..."
                    sudo zypper install -y gdb openocd
                    ;;
                *)
                    log_warning "Unknown distribution for debug tools installation"
                    ;;
            esac
        elif [[ "$OS" == "macos" ]]; then
            if command_exists brew; then
                log_info "Installing debug tools for macOS..."
                brew install gdb openocd
            fi
        fi
    else
        log_info "Debugging tools already installed"
    fi
}

# Verify installation
verify_installation() {
    log_info "Verifying installation..."

    local errors=0

    # Check Rust
    if command_exists rustc && command_exists cargo; then
        log_success "✓ Rust toolchain"
    else
        log_error "✗ Rust toolchain not found"
        errors=$((errors + 1))
    fi

    # Check Rust targets
    if rustup target list | grep -q "thumbv7m-none-eabi (installed)"; then
        log_success "✓ ARM Cortex-M target"
    else
        log_error "✗ ARM Cortex-M target not installed"
        errors=$((errors + 1))
    fi

    if rustup target list | grep -q "riscv32imac-unknown-none-elf (installed)"; then
        log_success "✓ RISC-V target"
    else
        log_error "✗ RISC-V target not installed"
        errors=$((errors + 1))
    fi

    # Check QEMU
    if command_exists qemu-system-arm; then
        log_success "✓ QEMU ARM support"
    else
        log_error "✗ QEMU ARM support not found"
        errors=$((errors + 1))
    fi

    if command_exists qemu-system-riscv32; then
        log_success "✓ QEMU RISC-V support"
    else
        log_error "✗ QEMU RISC-V support not found"
        errors=$((errors + 1))
    fi

    # Check optional tools
    if command_exists gdb; then
        log_success "✓ GDB (debugging)"
    else
        log_warning "⚠ GDB not found (optional for debugging)"
    fi

    if [[ $errors -eq 0 ]]; then
        log_success "All required dependencies installed successfully!"
        echo ""
        echo "🎉 karatOS development environment is ready!"
        echo ""
        echo "Next steps:"
        echo "  1. Clone or navigate to karatOS repository"
        echo "  2. Run: ./build.sh arm    # Test ARM build"
        echo "  3. Run: ./build.sh riscv  # Test RISC-V build"
        echo "  4. Run: ./qemu-arm.sh     # Run ARM demo"
        echo "  5. Run: ./qemu-riscv.sh   # Run RISC-V demo"
    else
        log_error "$errors required dependencies are missing"
        return 1
    fi
}

# Main installation function
main() {
    echo "========================================"
    echo "🚀 karatOS Dependency Installation Script"
    echo "========================================"
    echo ""

    # Detect OS
    detect_os

    # Check if running as root (not recommended for Rust installation)
    if [[ $EUID -eq 0 ]]; then
        log_warning "Running as root is not recommended for Rust installation"
        log_warning "Consider running as a regular user"
    fi

    # Install dependencies
    install_build_tools
    install_rust
    install_rust_targets
    install_qemu

    echo ""
    read -p "Install optional debugging tools? (y/N): " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        install_debug_tools
    fi

    echo ""
    verify_installation
}

# Run main function
main "$@"
