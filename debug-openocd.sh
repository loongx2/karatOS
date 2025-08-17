#!/bin/bash
# OpenOCD JTAG debugging setup for real hardware (optional)

echo "=== OpenOCD JTAG Debugging Setup ==="
echo "This script is for debugging on real ARM hardware with OpenOCD"
echo "For QEMU debugging, use ./qemu-jtag-debug.sh instead"
echo ""

# Check if OpenOCD is installed
if ! command -v openocd &> /dev/null; then
    echo "OpenOCD is not installed. Install with:"
    echo "  sudo apt install openocd"
    echo "  # or"
    echo "  brew install openocd"
    exit 1
fi

echo "Starting OpenOCD JTAG server..."
echo "This will create a GDB server on port 3333"
echo ""

# OpenOCD configuration for common ARM development boards
# Adjust the interface and target as needed for your hardware
openocd \
    -f interface/stlink.cfg \
    -f target/stm32f1x.cfg \
    -c "adapter speed 1000" \
    -c "transport select hla_swd" \
    -c "reset_config srst_only" \
    -c "init" \
    -c "reset halt"

echo ""
echo "OpenOCD JTAG session ended."
