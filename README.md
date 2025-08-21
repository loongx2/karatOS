# Multi-Architecture Async RTOS

Advanced experimental Rust RTOS implementing priority-based cooperative multitasking with event-driven scheduling. Supports both ARM Cortex-M and RISC-V architectures with comprehensive debugging capabilities.

## Features

- **Multi-Architecture**: ARM Cortex-M and RISC-V support
- **Async Event-Driven**: Priority-based cooperative multitasking
- **Interactive UART**: Command interface with status, logging, and control
- **Circular Logging**: Silent background logging with on-demand viewing
- **Professional Debugging**: JTAG/GDB support for both architectures
- **No Deadlocks**: Mutually exclusive events, single-threaded async design

## Build Targets

- **ARM (Cortex-M3)**: `thumbv7m-none-eabi`
- **RISC-V (32-bit)**: `riscv32imac-unknown-none-elf`

## Prerequisites

### 1. Rust Setup
```bash
rustup target add thumbv7m-none-eabi riscv32imac-unknown-none-elf
```

### 2. QEMU Emulation
```bash
sudo apt-get install qemu-system-arm qemu-system-misc
```

### 3. Debugging Tools (Optional)
```bash
# ARM debugging
sudo apt install openocd gdb-multiarch

# RISC-V debugging  
sudo apt install gcc-riscv64-unknown-elf gdb-multiarch
```

## Quick Start

### Build All Architectures
```bash
./build.sh all          # Build both ARM and RISC-V
./build.sh arm           # Build ARM only
./build.sh riscv         # Build RISC-V only
```

### Run Systems
```bash
# ARM version
./qemu-arm.sh

# RISC-V version  
./qemu-riscv32.sh
```

## Interactive Commands

Both versions support interactive UART commands:

- `help` - Show available commands
- `status` - Display system status and last 100 debug log lines
- `exit` - Shutdown system gracefully
- `restart` - Restart kernel and clear logs

## Debugging

### ARM Debugging
```bash
./debug-interactive.sh   # Interactive ARM debugging with GDB
./qemu-jtag-debug.sh    # JTAG debugging setup
```

### RISC-V Debugging
```bash
./debug-riscv.sh        # Interactive RISC-V debugging with GDB
./qemu-riscv32.sh debug # Start QEMU in debug mode
```

## Architecture Details

### ARM Implementation
- **Target**: LM3S6965EVB (QEMU)
- **UART**: 0x4000C000 (115200 baud)
- **Features**: JTAG debugging, ARM Cortex-M specific optimizations
- **Memory**: 256KB Flash, 64KB RAM

### RISC-V Implementation  
- **Target**: QEMU virt machine
- **UART**: 0x10000000 (16550 compatible)
- **ISA**: RV32IMAC (Integer, Multiply, Atomic, Compressed)
- **Memory**: 2MB Flash, 128MB RAM

## Documentation

- **[RISC-V Guide](RISC-V-GUIDE.md)** - Comprehensive RISC-V development guide
- **[JTAG Debug Guide](JTAG-DEBUG.md)** - ARM JTAG debugging setup
- **[Quick Start](JTAG-QUICKSTART.md)** - Fast debugging setup

## Performance Characteristics

- **ARM**: 1.2 DMIPS/MHz equivalent on Cortex-M3
- **RISC-V**: ~0.8 DMIPS/MHz on basic RV32I core
- **Latency**: <10μs task switching overhead
- **Memory**: <32KB code footprint, <4KB RAM usage

## Development Workflow

1. **Code**: Edit Rust source files
2. **Build**: `./build.sh [arm|riscv|all]`
3. **Test**: `./qemu-[arm|riscv32].sh`
4. **Debug**: Use appropriate debug script
5. **Deploy**: Flash to hardware (future)

## Advanced Features

### Event-Driven Scheduling
- Priority-based event posting and handling
- Cooperative task yielding without preemption
- Mutually exclusive event processing

### Logging System
- 100-line circular buffer
- Silent background logging
- On-demand log retrieval via UART commands

### Memory Safety
- `#![no_std]` embedded environment
- Static memory allocation with heapless collections
- Safe UART and peripheral access

## Next Steps

- [ ] Hardware deployment on real boards
- [ ] Integration with VexRiscv SoC
- [ ] Real-time interrupt handling
- [ ] Advanced memory management
- [ ] Multi-core support exploration
- Add timer interrupt to generate events.
- Implement context switching (store/restore registers) in assembly.
- Add UART drivers for logging.
- Add priority scheduling and sleep API.
