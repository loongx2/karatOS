# karatOS QEMU Testing Results

## Summary
Both ARM and RISC-V kernels successfully run in their respective QEMU environments with **real-time task scheduling demonstrations**.

## Test Results

### ✅ ARM Cortex-M3 (LM3S6965EVB) - Real-Time Scheduling
- **Target**: `thumbv7m-none-eabi`
- **QEMU Machine**: `lm3s6965evb` 
- **CPU**: `cortex-m3`
- **Memory Layout**: Flash at 0x00000000, RAM at 0x20000000
- **Entry Point**: ARM vector table with reset handler
- **Scheduling**: Round-robin with 4 tasks (High/Normal/Low/Event-Driven)
- **Output** (Live Task Execution):
  ```
  === karatOS Scheduler Example Starting ===
  Spawned Task 1 (High Priority) with ID: 1
  Spawned Task 2 (Normal Priority) with ID: 2
  Spawned Task 3 (Low Priority) with ID: 3
  Spawned Task 4 (Event-Driven) with ID: 4
  === All Tasks Spawned, Starting Round-Robin Scheduler ===

  Task 1 (High Priority): Counter = 507400 [Task 1 completed]
  Task 2 (Normal Priority): Processing data #507400 [Task 2 completed]
  Task 3 (Low Priority): Maintenance cycle 507400 [Task 3 completed]
  Task 4 (Event-Driven): Handling event 507400 [Task 4 completed]
  === Scheduler cycle: 20400 ===
  ```

### ✅ RISC-V (QEMU Virt Machine) - Real-Time Scheduling
- **Target**: `riscv32imac-unknown-none-elf`
- **QEMU Machine**: `virt`
- **CPU**: `rv32`
- **Memory Layout**: RAM at 0x80000000 (128MB)
- **Entry Point**: Custom `_start` function in `.text._start`
- **UART Output**: Direct memory-mapped UART at 0x10000000
- **Scheduling**: Round-robin with 4 tasks (High/Normal/Low/Event-Driven)
- **Output** (Live Task Execution):
  ```
  === karatOS Scheduler Example Starting ===
  Spawned Task 1 (High Priority) with ID: 1
  Spawned Task 2 (Normal Priority): Processing data #440700 [Task 2 completed]
  Task 3 (Low Priority): Maintenance cycle 440700 [Task 3 completed]
  Task 4 (Event-Driven): Handling event 440700 [Task 4 completed]
  === Scheduler cycle: 17650 ===
  ```

## Running the Tests

### Using the QEMU Test Script
```bash
# Run both architectures
./run-qemu.sh all

# Run specific architecture
./run-qemu.sh arm
./run-qemu.sh riscv
```

### Manual Commands
```bash
# ARM Cortex-M3
qemu-system-arm -machine lm3s6965evb -cpu cortex-m3 -nographic \
    -kernel kernel/target/thumbv7m-none-eabi/debug/kernel-arm-working

# RISC-V
qemu-system-riscv32 -machine virt -cpu rv32 -smp 1 -m 128M -nographic \
    -bios none -kernel kernel/target/riscv32imac-unknown-none-elf/debug/kernel-riscv-simple
```

> **⚠️ RECOMMENDED**: Use the provided bash scripts (`qemu-arm.sh` or `qemu-riscv.sh`) instead of manual QEMU commands. The scripts ensure:
> - Automatic kernel building with correct features
> - Proper QEMU arguments and machine configuration
> - Consistent testing environment
> - Latest kernel binary is always used

### Direct Script Usage
```bash
# ARM kernel
./qemu-arm.sh

# RISC-V kernel  
./qemu-riscv.sh
```

## Technical Details

### ARM Architecture
- Uses proper ARM Cortex-M vector table
- Implements reset handler and exception handlers
- Memory-mapped to LM3S6965EVB development board layout
- Uses cortex-m-rt and cortex-m-semihosting for platform support

### RISC-V Architecture  
- Custom boot sequence without relying on OpenSBI firmware
- Direct hardware initialization from `_start`
- UART communication via memory-mapped registers
- Simplified memory layout for QEMU virt machine

## Conclusion
Both ARM and RISC-V kernels demonstrate:
- ✅ Successful compilation for embedded targets
- ✅ Proper memory layout configuration
- ✅ Correct boot sequence execution
- ✅ Serial output functionality
- ✅ Stable infinite loop operation

The kernels are ready for further development and can serve as a foundation for a multi-architecture RTOS.
