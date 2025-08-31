# 🚀 Quick Reference Card

## One-Command Launch with Real-Time Scheduling

### Automated Setup (Recommended)
```bash
# Install all dependencies automatically
./install-dependencies.sh

# Then test everything
./test-platforms.sh
```

### Manual Setup
```bash
# ARM with round-robin task demo
./qemu-arm.sh

# RISC-V with round-robin task demo
./qemu-riscv.sh
```

> **🎯 NEW**: Both platforms demonstrate **real-time task scheduling** with live UART output showing task execution and counter increments.

## Expected Live Output (Both Platforms)

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

## Manual Commands (Advanced Users)

### ARM Platform ✅ (Real-Time Scheduling Demo)
```bash
# Build
./build.sh arm

# Run with live task scheduling
./qemu-arm.sh

# Expected live output:
# Task 1 (High Priority): Counter = 507400 [Task 1 completed]
# Task 2 (Normal Priority): Processing data #507400 [Task 2 completed]
# Task 3 (Low Priority): Maintenance cycle 507400 [Task 3 completed]
# Task 4 (Event-Driven): Handling event 507400 [Task 4 completed]
```

### RISC-V Platform ✅ (Real-Time Scheduling Demo)
```bash
# Build
./build.sh riscv

# Run with live task scheduling
./qemu-riscv.sh

# Expected live output:
# Task 1 (High Priority): Counter = 440700 [Task 1 completed]
# Task 2 (Normal Priority): Processing data #440700 [Task 2 completed]
# Task 3 (Low Priority): Maintenance cycle 440700 [Task 3 completed]
# Task 4 (Event-Driven): Handling event 440700 [Task 4 completed]
```

## Prerequisites

### Automated Installation (Recommended)
```bash
./install-dependencies.sh
```

### Manual Installation
```bash
# Install Rust targets
rustup target add riscv32imac-unknown-none-elf thumbv7m-none-eabi

# Install QEMU (Ubuntu/Debian)
sudo apt install qemu-system-arm qemu-system-riscv32

# Install QEMU (Fedora/RHEL)
sudo dnf install qemu-system-arm qemu-system-riscv32

# Install QEMU (Arch Linux)
sudo pacman -S qemu-system-arm qemu-system-riscv32

# Install QEMU (macOS)
brew install qemu
```

---
📖 **Full Guide**: [LAUNCH-GUIDE.md](LAUNCH-GUIDE.md)
