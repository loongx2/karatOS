#!/bin/bash
# Build and run ARM kernel on QEMU

# Ensure ARM target is installed
rustup target add thumbv7m-none-eabi

# Build the kernel with ARM features
cd kernel
cargo build --bin kernel --target thumbv7m-none-eabi --features arm

# Run in QEMU with semihosting enabled
qemu-system-arm -M lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel ../target/thumbv7m-none-eabi/debug/kernel
