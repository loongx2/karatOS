# RISC-V Minimal Kernel

A minimal RISC-V kernel for QEMU virt machine testing.

## Quick Start

```bash
# Build the kernel
./build.sh

# Run all tests
./test.sh

# Switch kernel variant
./kernel.sh debug
```

## Scripts

### `build.sh`
Clean build and verification of the RISC-V kernel.

### `test.sh`
Comprehensive test suite with multiple commands:
- `./test.sh test` - Run all automated tests
- `./test.sh direct` - Test direct kernel execution
- `./test.sh opensbi` - Test with OpenSBI firmware
- `./test.sh interactive` - Start interactive QEMU session
- `./test.sh debug` - Debug memory and registers
- `./test.sh dtb` - Show device tree information
- `./test.sh cleanup` - Clean up test artifacts

### `kernel.sh`
Kernel variant manager:
- `./kernel.sh current` - Show current kernel
- `./kernel.sh list` - List available variants
- `./kernel.sh [variant]` - Switch to variant (main, debug, blast, proper, test, verbose, memory)

## Kernel Variants

- **main** - Basic UART implementation
- **debug** - Debug version with memory markers
- **blast** - Simple UART blasting test
- **proper** - Properly initialized UART
- **test** - Multi-address UART testing
- **verbose** - Verbose output version
- **memory** - Memory verification test

## QEMU Configuration

Target: RISC-V 32-bit (`riscv32imac-unknown-none-elf`)
Machine: QEMU virt machine
Memory: 128MB RAM at 0x80000000
UART: NS16550A at 0x10000000

## Device Tree

The QEMU virt machine device tree is automatically generated as `virt.dtb`. 
Use `./test.sh dtb` to examine UART and memory configuration.

## Debugging

- Memory layout debugging: `./test.sh debug`
- Interactive session: `./test.sh interactive`
- GDB debugging: See parent directory configuration
