# 🚀 Quick Reference Card

## One-Command Launch

```bash
# Test everything
./test-platforms.sh

# RISC-V (Working)
cd kernel && ./kernel.sh test-riscv

# ARM (Build only)  
cd kernel && ./kernel.sh build-arm
```

## Manual Commands

### RISC-V Platform ✅
```bash
# Build
cargo build --target riscv32imac-unknown-none-elf --bin kernel-riscv-simple

# Run
qemu-system-riscv32 -machine virt -cpu rv32 -m 128M -nographic -bios none -kernel target/riscv32imac-unknown-none-elf/debug/kernel-riscv-simple

# Expected: "RISC-V kernel started!"
```

### ARM Platform ⚠️ 
```bash
# Build
cargo build --target thumbv7m-none-eabi --bin kernel --features arm

# Run (has issues)
qemu-system-arm -M lm3s6965evb -nographic -semihosting-config enable=on,target=native -serial mon:stdio -kernel target/thumbv7m-none-eabi/debug/kernel
```

## Prerequisites
```bash
rustup target add riscv32imac-unknown-none-elf thumbv7m-none-eabi
sudo apt install qemu-system-arm qemu-system-riscv32
```

---
📖 **Full Guide**: [LAUNCH-GUIDE.md](LAUNCH-GUIDE.md)
