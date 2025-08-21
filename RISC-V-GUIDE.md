# RISC-V Development Guide

This document describes how to build, run, and debug the RISC-V version of the async RTOS kernel.

## Prerequisites

### 1. Rust RISC-V Target
```bash
rustup target add riscv32imac-unknown-none-elf
```

### 2. QEMU RISC-V Support
```bash
# Ubuntu/Debian
sudo apt install qemu-system-misc

# Or build QEMU with RISC-V support
# Verify with: qemu-system-riscv32 -machine help
```

### 3. RISC-V Toolchain (for debugging)
```bash
# Option 1: Distribution packages
sudo apt install gcc-riscv64-unknown-elf gdb-multiarch

# Option 2: SiFive Toolchain
wget https://static.dev.sifive.com/dev-tools/riscv64-unknown-elf-gcc-8.3.0-2019.08.0-x86_64-linux-ubuntu14.tar.gz
tar -xzf riscv64-unknown-elf-gcc-8.3.0-2019.08.0-x86_64-linux-ubuntu14.tar.gz
export PATH=$PATH:$PWD/riscv64-unknown-elf-gcc-8.3.0-2019.08.0-x86_64-linux-ubuntu14/bin
```

## Building

### Quick Build
```bash
# Build RISC-V target
./build.sh riscv

# Or manually
cargo build -p kernel --target riscv32imac-unknown-none-elf --features riscv
```

### Build All Architectures
```bash
./build.sh all
```

## Running

### Basic Execution
```bash
./qemu-riscv32.sh
```

### Interactive Session
The RISC-V kernel includes UART command interface:
- `help` - Show available commands
- `status` - Display system status and logs
- `exit` - Shutdown system
- `restart` - Restart kernel

## Debugging

### GDB Debugging Session
```bash
# Terminal 1: Start QEMU in debug mode
./debug-riscv.sh

# Or manually:
./qemu-riscv32.sh debug
```

### GDB Commands
```gdb
# Connect to QEMU
target remote :1234

# Set RISC-V architecture
set architecture riscv:rv32

# Load debug symbols
file target/riscv32imac-unknown-none-elf/debug/kernel

# Useful debugging commands
info registers              # Show all registers
x/10i $pc                  # Disassemble 10 instructions at PC
break main                 # Break at main function
break *0x20000000         # Break at specific address
stepi                     # Step one instruction
continue                  # Continue execution
backtrace                 # Show call stack
```

### Memory Layout (QEMU virt machine)
```
0x00001000 - 0x00011fff   ROM (64KB)
0x10000000 - 0x10000fff   UART (16550 compatible)
0x10001000 - 0x10001fff   Virtio MMIO
0x80000000 - 0x87ffffff   RAM (128MB default)
0x20000000 - 0x201fffff   Flash (where kernel loads)
```

## Architecture Details

### RISC-V Features Used
- **ISA**: RV32IMAC (32-bit, Integer, Multiply/Divide, Atomic, Compressed)
- **Privilege**: Machine mode
- **ABI**: ilp32

### UART Implementation
- **Base Address**: 0x10000000 (QEMU virt machine)
- **Type**: 16550 compatible
- **Features**: TX/RX with status checking

### Memory Management
- **Flash Origin**: 0x20000000 (2MB)
- **RAM Origin**: 0x80000000 (128MB)
- **Stack**: Grows down from top of RAM
- **Heap**: Located after BSS section

## Comparison with ARM Version

| Feature | ARM (Cortex-M) | RISC-V |
|---------|----------------|---------|
| ISA | ARMv7-M | RV32IMAC |
| UART Base | 0x4000C000 | 0x10000000 |
| Memory Model | Harvard | Von Neumann |
| Registers | 16 general purpose | 32 general purpose |
| Calling Convention | AAPCS | RISC-V ABI |

## Troubleshooting

### Build Issues
```bash
# Missing target
rustup target add riscv32imac-unknown-none-elf

# Linker errors
cargo clean
cargo build -p kernel --target riscv32imac-unknown-none-elf --features riscv -v
```

### QEMU Issues
```bash
# Check QEMU RISC-V support
qemu-system-riscv32 -machine help | grep virt

# Check if binary is valid RISC-V ELF
file target/riscv32imac-unknown-none-elf/debug/kernel
```

### GDB Issues
```bash
# Use multiarch GDB if specific RISC-V GDB unavailable
gdb-multiarch target/riscv32imac-unknown-none-elf/debug/kernel

# Check GDB RISC-V support
gdb --batch --ex "set architecture" | grep riscv
```

## Development Workflow

1. **Edit Code**: Modify Rust source files
2. **Build**: `./build.sh riscv`
3. **Test**: `./qemu-riscv32.sh`
4. **Debug**: `./debug-riscv.sh` if issues found
5. **Repeat**: Iterate development cycle

## Future Enhancements

- [ ] Add RISC-V vector extensions support
- [ ] Implement RISC-V specific context switching
- [ ] Add RISC-V timer interrupt handling
- [ ] Support for RISC-V virtual memory
- [ ] Integration with VexRiscv SoC targets
- [ ] Performance profiling tools
