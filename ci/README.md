# karatOS CI/CD Documentation

## Overview

This document describes the comprehensive Docker-based CI/CD system for karatOS, featuring automated multi-architecture builds, QEMU testing, and Python orchestration.

## 🏗️ Architecture

### Components
- **Dockerfile**: Multi-stage build for ARM Cortex-M3 and RISC-V RV32IMAC
- **GitHub Actions**: Automated CI/CD pipeline with matrix builds  
- **Python Orchestrator**: `test_runner.py` for parallel build/test coordination
- **Docker Compose**: Local development and testing environment
- **Docker Management**: `docker_ci.py` for unified CI/CD operations

### Build Targets
- **ARM Cortex-M3**: `thumbv7m-none-eabi` (LM3S6965EVB board)
- **RISC-V RV32IMAC**: `riscv32imac-unknown-none-elf` (QEMU virt machine)

## 🚀 Quick Start

### Prerequisites
```bash
# Required tools
- Docker 20.10+
- Docker Compose 2.0+
- Python 3.8+ (for local orchestration)
```

### Local Development
```bash
# Make CI scripts executable
chmod +x ci/*.py

# Start interactive development environment
python3 ci/docker_ci.py shell

# Or use docker-compose directly
docker-compose run --rm karatos-ci
```

### CI/CD Pipeline
```bash
# Run complete CI/CD pipeline locally
python3 ci/docker_ci.py pipeline

# Run specific components
python3 ci/docker_ci.py build          # Build Docker images
python3 ci/docker_ci.py test           # Run tests
python3 ci/docker_ci.py release        # Create release artifacts
```

## 📋 Available Commands

### Docker CI Manager (`ci/docker_ci.py`)
```bash
# Core operations
python3 ci/docker_ci.py build [--service SERVICE]    # Build Docker images
python3 ci/docker_ci.py test                         # Run comprehensive tests
python3 ci/docker_ci.py pipeline                     # Complete CI/CD pipeline
python3 ci/docker_ci.py release                      # Generate release artifacts

# Development shortcuts  
python3 ci/docker_ci.py dev-arm                      # Quick ARM build
python3 ci/docker_ci.py dev-riscv                    # Quick RISC-V build
python3 ci/docker_ci.py dev-test                     # Quick test run

# Management
python3 ci/docker_ci.py shell [--service SERVICE]    # Interactive shell
python3 ci/docker_ci.py status                       # Service status
python3 ci/docker_ci.py logs [--service SERVICE]     # View logs
python3 ci/docker_ci.py cleanup                      # Clean up resources
```

### Test Runner (`ci/test_runner.py`)
```bash
# Run inside container or with local setup
python3 ci/test_runner.py --parallel --report        # Parallel build+test with report
python3 ci/test_runner.py --targets arm riscv        # Specific targets
python3 ci/test_runner.py --build-type release       # Release builds
```

### Docker Compose Services
```bash
# Individual services
docker-compose run --rm karatos-ci        # Interactive development
docker-compose run --rm karatos-arm       # ARM-only build
docker-compose run --rm karatos-riscv     # RISC-V-only build  
docker-compose run --rm karatos-test      # Comprehensive testing

# Parallel builds
docker-compose up karatos-arm karatos-riscv     # Run ARM and RISC-V in parallel
```

## 🔄 CI/CD Pipeline Details

### GitHub Actions Workflow (`.github/workflows/ci.yml`)

#### **Matrix Build Strategy**
```yaml
strategy:
  matrix:
    target: [arm, riscv]
    build_type: [debug, release]
```

#### **Pipeline Stages**
1. **Environment Validation**
   - Skip CI for documentation-only changes
   - Validate required tools and dependencies

2. **Multi-Architecture Build** 
   - Parallel ARM Cortex-M3 and RISC-V builds
   - Debug and release configurations
   - Binary size validation

3. **QEMU Testing**
   - 30-second timeout per architecture
   - Real-time scheduler validation
   - Performance metrics collection

4. **Integration Testing**
   - Python orchestrator validation
   - Comprehensive test report generation
   - Artifact collection

5. **Performance Regression**
   - Binary size limits enforcement
   - Performance baseline comparison
   - Automated failure detection

6. **Release Automation** (main branch only)
   - Automated tagging with timestamps
   - Release binary packaging
   - Checksum generation
   - GitHub release creation

### Performance Thresholds
```bash
# Binary size limits (enforced in CI)
ARM_DEBUG_MAX=900000     # ~900KB
ARM_RELEASE_MAX=30000    # ~30KB  
RISCV_DEBUG_MAX=950000   # ~950KB
RISCV_RELEASE_MAX=35000  # ~35KB
```

## 🐳 Docker Configuration

### Multi-Stage Dockerfile
```dockerfile
# Stages:
FROM rust:1.75-slim as base         # Base environment
FROM base as source                 # Source code layer
FROM source as builder              # Build execution
FROM builder as tester              # Testing layer
FROM scratch as artifacts           # Release artifacts
FROM source as development          # Development environment
```

### Docker Compose Services
- **karatos-ci**: Main development environment
- **karatos-arm**: ARM-specific builder
- **karatos-riscv**: RISC-V-specific builder  
- **karatos-test**: Testing orchestrator
- **karatos-docs**: Documentation generator
- **karatos-release**: Release artifact builder

### Volume Management
```yaml
volumes:
  cargo-cache:     # Persistent Cargo registry cache
  target-cache:    # Build target directory cache
  release-artifacts: # Release binary storage
```

## 📊 Test Reports

### JSON Report Format (`ci_report.json`)
```json
{
  "summary": {
    "total_builds": 4,
    "successful_builds": 4,
    "build_success_rate": "100.0%",
    "total_tests": 2,
    "successful_tests": 2,
    "test_success_rate": "100.0%"
  },
  "binary_sizes": {
    "arm_debug": 886816,
    "arm_release": 24996,
    "riscv_debug": 943060,
    "riscv_release": 33896
  },
  "builds": [...],
  "tests": [...]
}
```

### GitHub Actions Artifacts
- **Binary Artifacts**: `karatos-{target}-{build_type}` 
- **Test Reports**: `ci-test-report`
- **Release Packages**: Automated GitHub releases

## 🔧 Development Workflow

### Local Development Cycle
```bash
# 1. Start development environment
python3 ci/docker_ci.py shell

# 2. Make code changes (in container)
# Edit files in /workspace

# 3. Build and test specific target
./build.sh arm debug --verbose
./build.sh arm -t

# 4. Or run comprehensive testing
python3 ci/test_runner.py --parallel --report

# 5. Exit container and commit changes
exit
git add . && git commit -m "feature: description"
git push origin feature-branch
```

### CI/CD Integration
```bash
# Trigger GitHub Actions
git push origin main                    # Full CI pipeline

# Local CI validation  
python3 ci/docker_ci.py pipeline       # Run full pipeline locally

# Quick validation
python3 ci/docker_ci.py build          # Build images
python3 ci/docker_ci.py test           # Test builds
```

## 🎯 Performance Characteristics

### Build Performance
- **Parallel Builds**: ARM and RISC-V simultaneously
- **Docker Caching**: Layer caching for faster rebuilds
- **Cargo Caching**: Persistent dependency cache
- **Build Time**: ~2-3 minutes for full pipeline

### Resource Usage
- **Memory**: ~2GB for parallel builds
- **Storage**: ~1GB for caches and artifacts
- **CPU**: Efficient use of multi-core systems

### Test Execution
- **QEMU Testing**: 30s timeout per architecture
- **Parallel Testing**: Simultaneous ARM/RISC-V validation  
- **Comprehensive Validation**: Scheduler, binary size, performance

## 🚀 Release Process

### Automated Releases (Main Branch)
1. **Trigger**: Push to main branch
2. **Build**: Release binaries for ARM and RISC-V
3. **Package**: Descriptive binary names with checksums
4. **Tag**: Timestamp-based versioning (`v20250901-123456`)
5. **Release**: GitHub release with download links

### Manual Release
```bash
# Generate release artifacts locally
python3 ci/docker_ci.py release

# Artifacts created in ./release/
ls -la release/
# karatos-arm-cortex-m3.bin
# karatos-riscv-rv32imac.bin  
# checksums.sha256
```

## 🔍 Troubleshooting

### Common Issues

#### Docker Build Failures
```bash
# Clean rebuild
python3 ci/docker_ci.py cleanup
python3 ci/docker_ci.py build --service karatos-ci

# Check logs
python3 ci/docker_ci.py logs --service karatos-ci
```

#### QEMU Test Timeouts
```bash
# Extend timeout in docker-compose.yml
environment:
  - QEMU_TIMEOUT=60

# Or in GitHub Actions (ci.yml)
- name: Test with extended timeout
  run: QEMU_TIMEOUT=60 ./build.sh arm -t
```

#### Binary Size Regressions
```bash
# Check current sizes
python3 ci/test_runner.py --targets arm riscv --build-type release --report

# Adjust limits in .github/workflows/ci.yml
ARM_RELEASE_MAX=35000  # Increase if needed
```

### Debug Mode
```bash
# Enable verbose logging
LOG_LEVEL=debug python3 ci/docker_ci.py pipeline

# Interactive debugging
python3 ci/docker_ci.py shell
./build.sh arm debug --verbose
```

## 📈 Future Enhancements

### Planned Features
- **Hardware-in-the-Loop**: Real board testing integration
- **Performance Benchmarking**: Automated performance regression testing  
- **Multi-Platform**: Windows and macOS CI support
- **Cross-Platform Testing**: Additional RISC-V boards and ARM variants
- **Documentation**: Automated API documentation generation

### Integration Opportunities
- **Deployment**: Automated firmware deployment to development boards
- **Monitoring**: Real-time build and test metrics dashboard
- **Notifications**: Slack/Discord integration for build status
- **Security**: Automated vulnerability scanning and SBOM generation

---

**For questions or support, please open an issue in the karatOS repository.**
