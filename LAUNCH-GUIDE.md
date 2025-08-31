# 🚀 Multi-Platform Rust RTOS - Launch Guide

This guide provides comprehensive instructions for building and running the multi-platform Rust RTOS on both ARM and RISC-V architectures using QEMU emulation.

## 📋 Quick Start

### Prerequisites
```bash
# Install Rust targets
rustup target add thumbv7m-none-eabi        # ARM Cortex-M
rustup target add riscv32imac-unknown-none-elf  # RISC-V 32-bit

# Install QEMU (Ubuntu/Debian)
sudo apt install qemu-system-arm qemu-system-riscv32

# Install QEMU (macOS)
brew install qemu
```

### One-Command Test
```bash
# Test both platforms
./test-platforms.sh

# Expected output:
# ✅ RISC-V Platform: WORKING
# ✅ ARM Platform: BUILD SUCCESSFUL
```

## 🎯 Platform-Specific Commands

### RISC-V Platform (Fully Functional)

#### Build and Run
```bash
cd kernel
cargo build --target riscv32imac-unknown-none-elf --bin kernel-riscv-simple
qemu-system-riscv32 -machine virt -cpu rv32 -m 128M -nographic -bios none -kernel target/riscv32imac-unknown-none-elf/debug/kernel-riscv-simple
```

> **⚠️ IMPORTANT**: For consistent and reliable execution, always use the provided bash scripts (`qemu-arm.sh` or `qemu-riscv.sh`) instead of manual QEMU commands. The scripts automatically handle the build process and ensure correct QEMU configuration.

**Expected Output:**
```
RISC-V kernel started!
```

#### Quick Scripts
```bash
# Using the kernel script
cd kernel
./kernel.sh build-riscv     # Build RISC-V kernel
./kernel.sh run-riscv       # Run RISC-V kernel
./kernel.sh test-riscv      # Build and run

# Using QEMU scripts
./qemu/riscv/run.sh         # Build and run RISC-V
./qemu/riscv/debug.sh       # Run with debugging
```

### ARM Platform (Build Success)

#### Build and Run
```bash
cd kernel
cargo build --target thumbv7m-none-eabi --bin kernel --features arm
qemu-system-arm -M lm3s6965evb -nographic -semihosting-config enable=on,target=native -serial mon:stdio -kernel target/thumbv7m-none-eabi/debug/kernel
```

> **⚠️ IMPORTANT**: For consistent and reliable execution, always use the provided bash scripts (`qemu-arm.sh` or `qemu-riscv.sh`) instead of manual QEMU commands. The scripts automatically handle the build process and ensure correct QEMU configuration.

#### Quick Scripts
```bash
# Using the kernel script
cd kernel
./kernel.sh build-arm       # Build ARM kernel
./kernel.sh run-arm         # Run ARM kernel (has runtime issues)

# Using QEMU scripts
./qemu/arm/run.sh           # Build and run ARM
./qemu/arm/debug.sh         # Run with debugging
```

## 🛠️ Advanced Usage

### Development Scripts

#### Main Build Script
```bash
./build.sh                  # Interactive build menu
./build.sh arm              # Build ARM only
./build.sh riscv            # Build RISC-V only
./build.sh all              # Build both platforms
```

#### Testing Scripts
```bash
./test-all.sh               # Comprehensive test suite
./test-platforms.sh         # Platform validation
```

#### Debugging
```bash
# RISC-V debugging
./debug-riscv.sh            # RISC-V with GDB
./qemu/riscv/debug.sh       # RISC-V debug mode

# ARM debugging
./qemu/arm/debug.sh         # ARM debug mode
```

### Target-Specific Builds

#### RISC-V Variants
```bash
# Simple RISC-V kernel (recommended)
cargo build --target riscv32imac-unknown-none-elf --bin kernel-riscv-simple

# Full RISC-V kernel (with features)
cargo build --target riscv32imac-unknown-none-elf --bin kernel --features riscv
```

#### ARM Variants
```bash
# ARM with features
cargo build --target thumbv7m-none-eabi --bin kernel --features arm

# ARM for different Cortex-M variants
cargo build --target thumbv7em-none-eabihf --bin kernel --features arm  # Cortex-M4/M7
```

## 📁 Project Structure

```
rtos-rust/
├── kernel/                           # Main kernel source
│   ├── src/
│   │   ├── main.rs                  # Platform-agnostic kernel
│   │   ├── main_simple_riscv.rs     # Simple RISC-V kernel
│   │   ├── arch/                    # Architecture-specific code
│   │   ├── config/                  # Platform configurations
│   │   └── drivers/                 # Device drivers
│   ├── qemu/                        # QEMU launch scripts
│   │   ├── arm/                     # ARM-specific scripts
│   │   └── riscv/                   # RISC-V-specific scripts
│   ├── memory.x                     # Linker script
│   └── kernel.sh                    # Build and run script
├── test-platforms.sh                # Platform validation
├── build.sh                         # Interactive build menu
└── README.md                        # Project documentation
```

## 🔧 Configuration Files

### Cargo Configuration
- `.cargo/config.toml` - Rust target configuration
- `Cargo.toml` - Dependencies and features

### Linker Scripts
- `memory.x` - Main linker script (RISC-V optimized)
- `memory-arm.x` - ARM-specific linker script
- `memory-riscv.x` - RISC-V-specific linker script

### QEMU Configuration
- `riscv32imac-qemu-virt.json` - RISC-V target specification

## 🚨 Troubleshooting

### Common Issues

#### RISC-V Target Not Found
```bash
rustup target add riscv32imac-unknown-none-elf
```

#### ARM Target Not Found
```bash
rustup target add thumbv7m-none-eabi
```

#### QEMU Not Found
```bash
# Ubuntu/Debian
sudo apt install qemu-system-arm qemu-system-riscv32

# macOS
brew install qemu

# Arch Linux
sudo pacman -S qemu-arch-extra
```

#### Build Errors
```bash
# Clean build
cargo clean
cargo build --target riscv32imac-unknown-none-elf --bin kernel-riscv-simple

# Check toolchain
rustup show
```

### Platform Status

#### ✅ RISC-V Platform - Fully Functional
- **Build**: ✅ Success
- **QEMU Execution**: ✅ Working
- **Output**: ✅ "RISC-V kernel started!"
- **Stability**: ✅ Stable execution

#### ⚠️ ARM Platform - Build Success, Runtime Issues
- **Build**: ✅ Success
- **QEMU Execution**: ⚠️ Runtime configuration issues
- **Output**: ❌ Execution failures
- **Status**: Requires debugging for full functionality

## 📚 Additional Resources

- **RISC-V Guide**: `RISC-V-GUIDE.md` - Detailed RISC-V development guide
- **Test Results**: `kernel/TEST_RESULTS.md` - Comprehensive test documentation
- **JTAG Setup**: `JTAG-QUICKSTART.md` - Hardware debugging setup
- **Debug Guide**: `JTAG-DEBUG.md` - Advanced debugging techniques

## 🎯 Development Workflow

### Typical Development Session
```bash
# 1. Start development
cd rtos-rust

# 2. Test current status
./test-platforms.sh

# 3. Work on RISC-V (fully functional)
cd kernel
./kernel.sh test-riscv

# 4. Make changes
vim src/main_simple_riscv.rs

# 5. Test changes
cargo build --target riscv32imac-unknown-none-elf --bin kernel-riscv-simple
qemu-system-riscv32 -machine virt -cpu rv32 -m 128M -nographic -bios none -kernel target/riscv32imac-unknown-none-elf/debug/kernel-riscv-simple

# 6. Debug if needed
./qemu/riscv/debug.sh
```

### Performance Testing
```bash
# Build optimized release version
cargo build --target riscv32imac-unknown-none-elf --bin kernel-riscv-simple --release

# Run with performance monitoring
qemu-system-riscv32 -machine virt -cpu rv32 -m 128M -nographic -bios none -kernel target/riscv32imac-unknown-none-elf/release/kernel-riscv-simple -monitor stdio
```

## ✨ Key Features

- **Multi-Architecture Support**: ARM Cortex-M and RISC-V 32-bit
- **Platform Abstraction**: Unified kernel with architecture-specific drivers
- **QEMU Integration**: Complete emulation environment for both platforms
- **Comprehensive Testing**: Automated validation of both platforms
- **Modular Design**: Clean separation of platform-specific code
- **Development Tools**: Scripts for building, testing, and debugging

---

**🎉 Success Story**: The RISC-V platform has achieved full functional parity and now successfully boots, initializes, and outputs to console through QEMU emulation!
