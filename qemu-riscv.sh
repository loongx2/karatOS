#!/bin/bash
# Build and run RISC-V kernel on QEMU

# Ensure RISC-V target is installed
rustup target add riscv32imac-unknown-none-elf

# Build the kernel with RISC-V features
cd kernel
cargo build --bin kernel --target riscv32imac-unknown-none-elf --features riscv

# Run in QEMU virt machine with UART output
qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -smp 1 \
    -m 128M \
    -nographic \
    -bios none \
    -serial mon:stdio \
    -kernel ../target/riscv32imac-unknown-none-elf/debug/kernel
