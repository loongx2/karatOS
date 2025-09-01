#!/usr/bin/env python3
"""
karatOS CI/CD Test Runner
Python orchestrator for multi-architecture testing with Docker integration
"""

import os
import sys
import time
import json
import subprocess
import argparse
import logging
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('karatos-ci')

@dataclass
class BuildResult:
    """Build result container"""
    target: str
    build_type: str
    success: bool
    duration: float
    binary_size: int
    error_message: Optional[str] = None

@dataclass
class TestResult:
    """Test result container"""
    target: str
    success: bool
    duration: float
    output: str
    error_message: Optional[str] = None

class KaratOSCI:
    """Main CI/CD orchestrator for karatOS"""
    
    def __init__(self, workspace_dir: str = "."):
        self.workspace_dir = Path(workspace_dir)
        self.results = {
            'builds': [],
            'tests': [],
            'metrics': {}
        }
        
    def run_command(self, cmd: List[str], timeout: int = 300) -> Tuple[bool, str, float]:
        """Execute command with timeout and capture output"""
        start_time = time.time()
        try:
            logger.info(f"Executing: {' '.join(cmd)}")
            result = subprocess.run(
                cmd,
                cwd=self.workspace_dir,
                capture_output=True,
                text=True,
                timeout=timeout
            )
            duration = time.time() - start_time
            
            if result.returncode == 0:
                logger.info(f"Command succeeded in {duration:.2f}s")
                return True, result.stdout, duration
            else:
                logger.error(f"Command failed: {result.stderr}")
                return False, result.stderr, duration
                
        except subprocess.TimeoutExpired:
            duration = time.time() - start_time
            logger.error(f"Command timed out after {timeout}s")
            return False, f"Timeout after {timeout}s", duration
        except Exception as e:
            duration = time.time() - start_time
            logger.error(f"Command error: {e}")
            return False, str(e), duration

    def build_target(self, target: str, build_type: str = "debug") -> BuildResult:
        """Build specific target (arm/riscv)"""
        logger.info(f"Building {target} ({build_type})")
        
        cmd = ["./build.sh", target, build_type, "--verbose"]
        success, output, duration = self.run_command(cmd, timeout=120)
        
        # Get binary size if build succeeded
        binary_size = 0
        if success:
            binary_size = self._get_binary_size(target, build_type)
            
        result = BuildResult(
            target=target,
            build_type=build_type,
            success=success,
            duration=duration,
            binary_size=binary_size,
            error_message=None if success else output
        )
        
        self.results['builds'].append(result)
        return result

    def test_target(self, target: str) -> TestResult:
        """Test specific target with QEMU"""
        logger.info(f"Testing {target} with QEMU")
        
        cmd = ["./build.sh", target, "-t"]
        success, output, duration = self.run_command(cmd, timeout=60)
        
        result = TestResult(
            target=target,
            success=success,
            duration=duration,
            output=output,
            error_message=None if success else output
        )
        
        self.results['tests'].append(result)
        return result

    def _get_binary_size(self, target: str, build_type: str) -> int:
        """Get binary size for target"""
        target_map = {
            'arm': 'thumbv7m-none-eabi',
            'riscv': 'riscv32imac-unknown-none-elf'
        }
        
        triple = target_map.get(target)
        if not triple:
            return 0
            
        binary_path = self.workspace_dir / f"target/{triple}/{build_type}/kernel"
        
        try:
            return binary_path.stat().st_size
        except FileNotFoundError:
            logger.warning(f"Binary not found: {binary_path}")
            return 0

    def run_parallel_builds(self, targets: List[str], build_type: str = "debug") -> List[BuildResult]:
        """Run builds in parallel"""
        logger.info(f"Starting parallel builds for {targets} ({build_type})")
        
        with ThreadPoolExecutor(max_workers=len(targets)) as executor:
            future_to_target = {
                executor.submit(self.build_target, target, build_type): target 
                for target in targets
            }
            
            results = []
            for future in as_completed(future_to_target):
                target = future_to_target[future]
                try:
                    result = future.result()
                    results.append(result)
                    
                    if result.success:
                        logger.info(f"✅ {target} build succeeded ({result.binary_size} bytes)")
                    else:
                        logger.error(f"❌ {target} build failed: {result.error_message}")
                        
                except Exception as e:
                    logger.error(f"❌ {target} build exception: {e}")
                    
        return results

    def run_parallel_tests(self, targets: List[str]) -> List[TestResult]:
        """Run tests in parallel"""
        logger.info(f"Starting parallel tests for {targets}")
        
        with ThreadPoolExecutor(max_workers=len(targets)) as executor:
            future_to_target = {
                executor.submit(self.test_target, target): target 
                for target in targets
            }
            
            results = []
            for future in as_completed(future_to_target):
                target = future_to_target[future]
                try:
                    result = future.result()
                    results.append(result)
                    
                    if result.success:
                        logger.info(f"✅ {target} tests passed ({result.duration:.2f}s)")
                    else:
                        logger.error(f"❌ {target} tests failed: {result.error_message}")
                        
                except Exception as e:
                    logger.error(f"❌ {target} test exception: {e}")
                    
        return results

    def validate_environment(self) -> bool:
        """Validate CI environment"""
        logger.info("Validating CI environment")
        
        # Check required tools
        tools = ['rustc', 'cargo', 'qemu-system-arm', 'qemu-system-riscv32']
        for tool in tools:
            success, _, _ = self.run_command(['which', tool], timeout=10)
            if not success:
                logger.error(f"Required tool not found: {tool}")
                return False
                
        # Check Rust targets
        success, output, _ = self.run_command(['rustup', 'target', 'list', '--installed'], timeout=10)
        if not success:
            logger.error("Failed to check Rust targets")
            return False
            
        required_targets = ['thumbv7m-none-eabi', 'riscv32imac-unknown-none-elf']
        for target in required_targets:
            if target not in output:
                logger.error(f"Required Rust target not installed: {target}")
                return False
                
        logger.info("✅ Environment validation passed")
        return True

    def generate_report(self) -> Dict:
        """Generate comprehensive test report"""
        total_builds = len(self.results['builds'])
        successful_builds = sum(1 for b in self.results['builds'] if b.success)
        
        total_tests = len(self.results['tests'])
        successful_tests = sum(1 for t in self.results['tests'] if t.success)
        
        # Calculate metrics
        build_success_rate = (successful_builds / total_builds * 100) if total_builds > 0 else 0
        test_success_rate = (successful_tests / total_tests * 100) if total_tests > 0 else 0
        
        # Binary sizes
        binary_sizes = {}
        for build in self.results['builds']:
            if build.success:
                binary_sizes[f"{build.target}_{build.build_type}"] = build.binary_size
        
        report = {
            'summary': {
                'total_builds': total_builds,
                'successful_builds': successful_builds,
                'build_success_rate': f"{build_success_rate:.1f}%",
                'total_tests': total_tests,
                'successful_tests': successful_tests,
                'test_success_rate': f"{test_success_rate:.1f}%"
            },
            'binary_sizes': binary_sizes,
            'builds': [
                {
                    'target': b.target,
                    'build_type': b.build_type,
                    'success': b.success,
                    'duration': f"{b.duration:.2f}s",
                    'binary_size': b.binary_size,
                    'error': b.error_message
                }
                for b in self.results['builds']
            ],
            'tests': [
                {
                    'target': t.target,
                    'success': t.success,
                    'duration': f"{t.duration:.2f}s",
                    'error': t.error_message
                }
                for t in self.results['tests']
            ]
        }
        
        return report

    def save_report(self, report: Dict, filename: str = "ci_report.json"):
        """Save report to file"""
        report_path = self.workspace_dir / filename
        with open(report_path, 'w') as f:
            json.dump(report, f, indent=2)
        logger.info(f"Report saved to {report_path}")

def main():
    parser = argparse.ArgumentParser(description='karatOS CI/CD Test Runner')
    parser.add_argument('--targets', nargs='+', default=['arm', 'riscv'],
                       help='Targets to build and test')
    parser.add_argument('--build-type', default='debug',
                       choices=['debug', 'release'],
                       help='Build type')
    parser.add_argument('--parallel', action='store_true',
                       help='Run builds and tests in parallel')
    parser.add_argument('--report', action='store_true',
                       help='Generate JSON report')
    parser.add_argument('--workspace', default='.',
                       help='Workspace directory')
    
    args = parser.parse_args()
    
    # Initialize CI runner
    ci = KaratOSCI(args.workspace)
    
    # Validate environment
    if not ci.validate_environment():
        logger.error("Environment validation failed")
        sys.exit(1)
    
    # Run builds
    if args.parallel:
        build_results = ci.run_parallel_builds(args.targets, args.build_type)
    else:
        build_results = [ci.build_target(target, args.build_type) for target in args.targets]
    
    # Check if all builds succeeded
    if not all(r.success for r in build_results):
        logger.error("Some builds failed, skipping tests")
        if args.report:
            report = ci.generate_report()
            ci.save_report(report)
        sys.exit(1)
    
    # Run tests
    if args.parallel:
        test_results = ci.run_parallel_tests(args.targets)
    else:
        test_results = [ci.test_target(target) for target in args.targets]
    
    # Generate report
    if args.report:
        report = ci.generate_report()
        ci.save_report(report)
        
        # Print summary
        print("\n" + "="*60)
        print("CI/CD SUMMARY")
        print("="*60)
        print(f"Builds: {report['summary']['successful_builds']}/{report['summary']['total_builds']} "
              f"({report['summary']['build_success_rate']})")
        print(f"Tests:  {report['summary']['successful_tests']}/{report['summary']['total_tests']} "
              f"({report['summary']['test_success_rate']})")
        
        print("\nBinary Sizes:")
        for name, size in report['binary_sizes'].items():
            print(f"  {name}: {size:,} bytes ({size/1024:.1f} KB)")
    
    # Exit with error if any tests failed
    if not all(r.success for r in test_results):
        logger.error("Some tests failed")
        sys.exit(1)
    
    logger.info("🎉 All builds and tests passed!")

if __name__ == '__main__':
    main()
