# Local CI/CD Testing Guide

## 🎯 Testing GitHub Actions Locally

### **Available Testing Methods**

### **Python Orchestrator (Recommended)**
```bash
# Fastest local testing - mirrors GitHub Actions workflow
python3 ci/test_runner.py --targets arm riscv --build-type debug --parallel --report

# Test specific targets
python3 ci/test_runner.py --targets arm --build-type release --report
python3 ci/test_runner.py --targets riscv --build-type debug --report
```

### **Direct Build Script Testing**
```bash
# Quick individual builds with testing
./build.sh arm debug --test
./build.sh riscv release --test

# Test all targets
./build.sh all debug --test
./build.sh all release --test
```

### **Docker-based Testing**
```bash
# Use Docker CI manager (if docker-compose available)
python3 ci/docker_ci.py test
python3 ci/docker_ci.py pipeline
```

## 📊 Expected Test Results

✅ **Binary Size Targets:**
- ARM Debug: ~886KB (under 900KB limit)
- ARM Release: ~24KB (under 30KB limit)
- RISC-V Debug: ~943KB (under 950KB limit)
- RISC-V Release: ~33KB (under 40KB limit)

✅ **All QEMU tests should pass with 30s timeout**

✅ **Python orchestrator provides detailed reporting**

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
