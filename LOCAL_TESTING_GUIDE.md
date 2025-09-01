# Local CI/CD Testing Guide

## 🎯 Testing GitHub Actions Locally

### **Available Testing Methods**

### **Python Orchestrator (Recommended)**
```bash
# Test release builds (matches GitHub Actions) with clean state
python3 ci/test_runner.py --targets arm riscv --build-type release --parallel --report

# Test specific targets with clean builds
python3 ci/test_runner.py --targets arm --build-type release --report
python3 ci/test_runner.py --targets riscv --build-type release --report
```

### **Direct Build Script Testing**
```bash
# Test release builds with clean state (matches GitHub Actions exactly)
./build.sh arm release --clean --test
./build.sh riscv release --clean --test

# Test all targets with clean state
./build.sh all release --clean --test
```

### **Docker-based Testing**
```bash
# Use Docker CI manager (if docker-compose available)
python3 ci/docker_ci.py test
python3 ci/docker_ci.py pipeline
```

## 📊 Expected Test Results

✅ **Release Binary Size Targets (GitHub Actions only tests these):**
- ARM Release: ~25KB (under 30KB limit)
- RISC-V Release: ~34KB (under 35KB limit)

✅ **All QEMU tests should pass with 30s timeout**

✅ **Python orchestrator provides detailed reporting**

## 🚀 GitHub Actions Status
The CI/CD pipeline now only tests release builds with clean state for:
- Faster CI execution
- More reliable builds
- Focus on production-ready binaries

## 🚀 Workflow

1. **Make code changes**
2. **Run local tests:** `./test-github-actions.sh`
3. **If tests pass:** `git add . && git commit -m "message"`
4. **Pre-commit hook runs automatically**
5. **If all good:** `git push origin main`

## 🔧 Scripts Created

- `test-github-actions.sh` - Full CI/CD simulation
- `test-act-lightweight.sh` - Quick syntax check with Act
- `.git/hooks/pre-commit` - Automatic pre-commit testing

## 💡 Benefits

- **Zero failed CI/CD runs on GitHub** 🎯
- **Catch issues locally before pushing** 🛡️
- **Save GitHub Actions minutes** 💰
- **Faster development cycle** ⚡
- **Confidence in every push** 🚀
# Test change
