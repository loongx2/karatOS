RTOS Prototype
==============

Minimal experimental Rust RTOS skeleton targeting ARM Cortex-M (via QEMU) and RISC-V (virt) for demonstration.

Build Targets
------------

ARM (Cortex-M3 example): thumbv7m-none-eabi
RISC-V (32-bit embedded): riscv32imac-unknown-none-elf

Prerequisites
-------------
1. Rust nightly (recommended for inline asm) and target install:
```
rustup target add thumbv7m-none-eabi riscv32imac-unknown-none-elf
```
2. QEMU with arm and riscv support:
```
sudo apt-get install qemu-system-arm qemu-system-misc
```

Building
--------
ARM:
```
cargo build -p kernel --target thumbv7m-none-eabi --features arm
```
RISC-V:
```
cargo build -p kernel --target riscv32imac-unknown-none-elf --features riscv
```

Running under QEMU
------------------
ARM (Cortex-M3 QEMU board 'lm3s6965evb'):
```
qemu-system-arm -M lm3s6965evb -nographic -kernel target/thumbv7m-none-eabi/debug/kernel
```
RISC-V (virt machine):
```
qemu-system-riscv32 -M virt -nographic -kernel target/riscv32imac-unknown-none-elf/debug/kernel
```

Output / Logging
----------------
Currently no UART init; to see output you'd add semihosting (ARM) or simple MMIO UART writes (RISC-V). This skeleton focuses on structure; extend `arch` modules for real I/O.

Next Steps
----------
- Add timer interrupt to generate events.
- Implement context switching (store/restore registers) in assembly.
- Add UART drivers for logging.
- Add priority scheduling and sleep API.
