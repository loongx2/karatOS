#!/bin/bash
# karatOS Architecture-Agnostic Test Runner
# Tests both ARM and RISC-V implementations and compares results

set -e  # Exit on any error

echo "=== karatOS Cross-Architecture Test Runner ==="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
TEST_TIMEOUT=30  # seconds - increased for parallel execution
QEMU_TIMEOUT=10  # seconds for QEMU to run and produce output
ARM_LOG="arm_test.log"
RISCV_LOG="riscv_test.log"
COMPARISON_LOG="comparison.log"

# Function to run test on specific architecture
run_architecture_test() {
    local arch=$1
    local log_file=$2
    local target=$3
    local qemu_machine=$4
    local kernel_binary=$5

    echo -e "${YELLOW}Testing $arch architecture...${NC}"

    # Clean previous log
    rm -f "$log_file"

    # Run QEMU and capture output in background
    if [ "$target" = "arm" ]; then
        timeout $QEMU_TIMEOUT qemu-system-$target \
            -M $qemu_machine \
            -nographic \
            -semihosting-config enable=on,target=native \
            -serial mon:stdio \
            -kernel "$kernel_binary" > "$log_file" 2>&1 &
    else
        timeout $QEMU_TIMEOUT qemu-system-$target \
            -machine $qemu_machine \
            -cpu rv32 \
            -smp 1 \
            -m 128M \
            -nographic \
            -bios none \
            -serial mon:stdio \
            -kernel "$kernel_binary" > "$log_file" 2>&1 &
    fi
}

# Function to extract test results from log
extract_test_results() {
    local log_file=$1
    local arch=$2

    echo "=== $arch Test Results ===" > "$log_file.results"

    # Extract key test output lines
    grep -E "(Test [0-9]+:|===|Events Processed:|Tasks Executed:|Scheduler Cycles:|Priority Switches:)" "$log_file" >> "$log_file.results" 2>/dev/null || echo "No test results found in $log_file" >> "$log_file.results"

    cat "$log_file.results"
}

# Function to compare results
compare_results() {
    local arm_results="arm_test.log.results"
    local riscv_results="riscv_test.log.results"

    echo "=== Cross-Architecture Test Comparison ===" > "$COMPARISON_LOG"
    echo "Comparing ARM and RISC-V test results..." >> "$COMPARISON_LOG"
    echo "" >> "$COMPARISON_LOG"

    # Check if both result files exist
    if [ ! -f "$arm_results" ] || [ ! -f "$riscv_results" ]; then
        echo -e "${RED}Error: Test result files not found${NC}" >> "$COMPARISON_LOG"
        return 1
    fi

    # Compare key metrics
    local arm_events=$(grep "Events Processed:" "$arm_results" | wc -l)
    local riscv_events=$(grep "Events Processed:" "$riscv_results" | wc -l)

    local arm_tasks=$(grep "Tasks Executed:" "$arm_results" | wc -l)
    local riscv_tasks=$(grep "Tasks Executed:" "$riscv_results" | wc -l)

    local arm_cycles=$(grep "Scheduler Cycles:" "$arm_results" | wc -l)
    local riscv_cycles=$(grep "Scheduler Cycles:" "$riscv_results" | wc -l)

    local arm_switches=$(grep "Priority Switches:" "$arm_results" | wc -l)
    local riscv_switches=$(grep "Priority Switches:" "$riscv_results" | wc -l)

    echo "ARM Events: $arm_events, RISC-V Events: $riscv_events" >> "$COMPARISON_LOG"
    echo "ARM Tasks: $arm_tasks, RISC-V Tasks: $riscv_tasks" >> "$COMPARISON_LOG"
    echo "ARM Cycles: $arm_cycles, RISC-V Cycles: $riscv_cycles" >> "$COMPARISON_LOG"
    echo "ARM Switches: $arm_switches, RISC-V Switches: $arm_switches" >> "$COMPARISON_LOG"
    echo "" >> "$COMPARISON_LOG"

    # Check for consistency
    local differences=0

    if [ "$arm_events" != "$riscv_events" ]; then
        echo "WARNING: Different number of events processed" >> "$COMPARISON_LOG"
        differences=$((differences + 1))
    fi

    if [ "$arm_tasks" != "$riscv_tasks" ]; then
        echo "WARNING: Different number of tasks executed" >> "$COMPARISON_LOG"
        differences=$((differences + 1))
    fi

    if [ "$arm_cycles" != "$riscv_cycles" ]; then
        echo "WARNING: Different scheduler cycles" >> "$COMPARISON_LOG"
        differences=$((differences + 1))
    fi

    if [ "$arm_switches" != "$riscv_switches" ]; then
        echo "WARNING: Different priority switches" >> "$COMPARISON_LOG"
        differences=$((differences + 1))
    fi

    if [ $differences -eq 0 ]; then
        echo -e "${GREEN}SUCCESS: All metrics match between architectures${NC}" >> "$COMPARISON_LOG"
        echo -e "${GREEN}✓ Architecture-agnostic test PASSED${NC}"
    else
        echo -e "${RED}FAILURE: $differences differences found between architectures${NC}" >> "$COMPARISON_LOG"
        echo -e "${RED}✗ Architecture-agnostic test FAILED${NC}"
    fi

    echo "" >> "$COMPARISON_LOG"
    echo "=== Detailed Results ===" >> "$COMPARISON_LOG"
    echo "ARM Results:" >> "$COMPARISON_LOG"
    cat "$arm_results" >> "$COMPARISON_LOG"
    echo "" >> "$COMPARISON_LOG"
    echo "RISC-V Results:" >> "$COMPARISON_LOG"
    cat "$riscv_results" >> "$COMPARISON_LOG"
}

# Main test execution
echo "Building test versions for both architectures..."

# Build ARM test version
echo "Building ARM test kernel..."
cd kernel
cargo build --features "arm test_mode" --target thumbv7m-none-eabi
cd ..

# Build RISC-V test version
echo "Building RISC-V test kernel..."
cd kernel
cargo build --features "riscv test_mode" --target riscv32imac-unknown-none-elf
cd ..

# Run tests on both architectures in parallel
echo -e "${YELLOW}Running tests in parallel...${NC}"
run_architecture_test "ARM" "$ARM_LOG" "arm" "lm3s6965evb" "target/thumbv7m-none-eabi/debug/kernel"
run_architecture_test "RISC-V" "$RISCV_LOG" "riscv32" "virt" "target/riscv32imac-unknown-none-elf/debug/kernel"

# Wait for both tests to complete
echo -e "${YELLOW}Waiting for tests to complete...${NC}"
wait

# Check results
if [ -s "$ARM_LOG" ]; then
    echo -e "${GREEN}ARM test completed. Output saved to $ARM_LOG${NC}"
else
    echo -e "${RED}ARM test completed but no output captured${NC}"
fi

if [ -s "$RISCV_LOG" ]; then
    echo -e "${GREEN}RISC-V test completed. Output saved to $RISCV_LOG${NC}"
else
    echo -e "${RED}RISC-V test completed but no output captured${NC}"
fi

# Extract and compare results
echo -e "${YELLOW}Extracting test results...${NC}"
extract_test_results "$ARM_LOG" "ARM"
extract_test_results "$RISCV_LOG" "RISC-V"

echo -e "${YELLOW}Comparing results...${NC}"
compare_results

# Display comparison summary
echo ""
echo "=== Test Summary ==="
cat "$COMPARISON_LOG"

echo ""
echo "Detailed logs saved to:"
echo "  ARM: $ARM_LOG"
echo "  RISC-V: $RISCV_LOG"
echo "  Comparison: $COMPARISON_LOG"
