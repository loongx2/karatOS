# Dockerfile for karatOS Multi-Architecture CI/CD
# Multi-stage build optimized for ARM Cortex-M3 and RISC-V RV32IMAC

# Stage 1: Base Build Environment
FROM rust:1.75-slim as base
LABEL maintainer="karatOS Contributors"
LABEL description="karatOS Multi-Architecture Build Environment"

# Install system dependencies
RUN apt-get update && apt-get install -y \
    qemu-system-arm \
    qemu-system-riscv32 \
    qemu-system-misc \
    build-essential \
    git \
    curl \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install Rust targets
RUN rustup target add thumbv7m-none-eabi riscv32imac-unknown-none-elf

# Install Python dependencies for CI orchestration
RUN pip3 install --no-cache-dir \
    pyyaml \
    toml \
    requests \
    pytest

# Create workspace
WORKDIR /workspace

# Stage 2: Source and Dependencies
FROM base as source
COPY Cargo.toml Cargo.lock ./
COPY kernel/ ./kernel/
COPY build/ ./build/
COPY *.sh ./

# Make scripts executable
RUN chmod +x *.sh

# Stage 3: Build karatOS
FROM source as builder

# Set environment variables for reproducible builds
ENV CARGO_HOME=/usr/local/cargo
ENV RUST_BACKTRACE=1
ENV CARGO_INCREMENTAL=0

# Build debug versions (for testing)
RUN ./build.sh all debug --verbose

# Build release versions (for deployment)
RUN ./build.sh all release --verbose

# Validate builds
RUN ls -la target/thumbv7m-none-eabi/release/kernel target/riscv32imac-unknown-none-elf/release/kernel

# Stage 4: Testing
FROM builder as tester

# Copy test orchestration script
COPY ci/test_runner.py ./ci/

# Run comprehensive tests
RUN python3 ci/test_runner.py --parallel --report

# Stage 5: Artifacts (for production)
FROM scratch as artifacts
COPY --from=builder /workspace/target/thumbv7m-none-eabi/release/kernel /arm-kernel
COPY --from=builder /workspace/target/riscv32imac-unknown-none-elf/release/kernel /riscv-kernel

# Stage 6: Development Environment (optional)
FROM source as development
RUN ./install-dependencies.sh --assume-yes
CMD ["/bin/bash"]

# Default stage for CI/CD
FROM builder as default
CMD ["python3", "ci/test_runner.py"]
