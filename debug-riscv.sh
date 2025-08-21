#!/bin/bash

# RISC-V GDB Debug Script
# Launches QEMU with debug support and connects GDB

set -e

KERNEL_PATH="target/riscv32imac-unknown-none-elf/debug/kernel"
GDB_PORT="1234"

echo "=== RISC-V Debug Session Launcher ==="

# Check if kernel exists
if [ ! -f "$KERNEL_PATH" ]; then
    echo "Error: Debug kernel not found at $KERNEL_PATH"
    echo "Please build debug version first:"
    echo "  cargo build --target riscv32imac-unknown-none-elf --features riscv"
    exit 1
fi

# Check if riscv32-unknown-elf-gdb is available
if ! command -v riscv32-unknown-elf-gdb &> /dev/null && ! command -v riscv64-unknown-elf-gdb &> /dev/null; then
    echo "Warning: RISC-V GDB not found in PATH"
    echo "You may need to install a RISC-V toolchain:"
    echo "  # Ubuntu/Debian:"
    echo "  sudo apt install gcc-riscv64-unknown-elf gdb-multiarch"
    echo "  # Or download from SiFive tools"
    echo ""
fi

# Create temporary GDB init script
GDB_INIT_FILE=$(mktemp /tmp/gdb_init_riscv.XXXXXX)
cat > "$GDB_INIT_FILE" << 'EOF'
# RISC-V GDB initialization script

# Connect to QEMU
target remote :1234

# Set architecture to RISC-V 32-bit
set architecture riscv:rv32

# Load debug symbols
file target/riscv32imac-unknown-none-elf/debug/kernel

# Set some useful breakpoints
# break _start
# break main
# break arch_init

# Display useful information
info registers
info target

# Useful commands:
# (gdb) stepi          - Step one instruction
# (gdb) continue       - Continue execution  
# (gdb) info registers - Show all registers
# (gdb) x/10i $pc      - Disassemble 10 instructions at PC
# (gdb) backtrace      - Show call stack
# (gdb) break *0x...   - Set breakpoint at address
# (gdb) watch variable - Watch variable changes

echo "\n=== RISC-V Debug Session Ready ==="
echo "Use 'continue' to start execution"
echo "Use 'stepi' for single-step debugging"
echo "Use 'info registers' to see register state"
echo "Use 'help' for more GDB commands"
EOF

echo "Starting QEMU with debug support..."
echo "QEMU will wait for GDB connection on port $GDB_PORT"

# Start QEMU in background with debug support
qemu-system-riscv32 \
    -machine virt \
    -cpu rv32 \
    -smp 1 \
    -m 128M \
    -nographic \
    -serial stdio \
    -kernel "$KERNEL_PATH" \
    -s -S &

QEMU_PID=$!

# Give QEMU a moment to start
sleep 2

echo "Starting GDB..."

# Try different RISC-V GDB variants
if command -v riscv32-unknown-elf-gdb &> /dev/null; then
    GDB_CMD="riscv32-unknown-elf-gdb"
elif command -v riscv64-unknown-elf-gdb &> /dev/null; then
    GDB_CMD="riscv64-unknown-elf-gdb"
elif command -v gdb-multiarch &> /dev/null; then
    GDB_CMD="gdb-multiarch"
else
    echo "No suitable GDB found, trying generic gdb..."
    GDB_CMD="gdb"
fi

echo "Using GDB: $GDB_CMD"

# Launch GDB with our init script
"$GDB_CMD" -x "$GDB_INIT_FILE"

# Cleanup
echo "Cleaning up..."
kill $QEMU_PID 2>/dev/null || true
rm -f "$GDB_INIT_FILE"

echo "Debug session ended."
