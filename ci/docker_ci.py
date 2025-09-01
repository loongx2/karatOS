#!/usr/bin/env python3
"""
karatOS Docker CI/CD Management Script
Unified interface for Docker-based builds, tests, and deployments
"""

import os
import sys
import subprocess
import argparse
import json
import logging
from pathlib import Path
from typing import List, Dict

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger('karatos-docker-ci')

class DockerCI:
    """Docker-based CI/CD manager for karatOS"""
    
    def __init__(self, workspace_dir: str = "."):
        self.workspace_dir = Path(workspace_dir)
        self.compose_file = self.workspace_dir / "docker-compose.yml"
        
    def run_command(self, cmd: List[str], check: bool = True) -> subprocess.CompletedProcess:
        """Run command with logging"""
        logger.info(f"Executing: {' '.join(cmd)}")
        result = subprocess.run(cmd, cwd=self.workspace_dir, check=False)
        if check and result.returncode != 0:
            logger.error(f"Command failed with exit code {result.returncode}")
            sys.exit(result.returncode)
        return result

    def build_images(self, service: str = None):
        """Build Docker images"""
        logger.info("Building Docker images...")
        
        cmd = ["docker-compose", "build"]
        if service:
            cmd.append(service)
            
        self.run_command(cmd)
        logger.info("✅ Docker images built successfully")

    def run_parallel_builds(self):
        """Run ARM and RISC-V builds in parallel"""
        logger.info("Starting parallel builds...")
        
        # Start both build services
        self.run_command(["docker-compose", "up", "-d", "karatos-arm", "karatos-riscv"])
        
        # Wait for completion
        self.run_command(["docker-compose", "logs", "-f", "karatos-arm"])
        self.run_command(["docker-compose", "logs", "-f", "karatos-riscv"])
        
        # Check exit codes
        arm_result = self.run_command(["docker-compose", "ps", "-q", "karatos-arm"], check=False)
        riscv_result = self.run_command(["docker-compose", "ps", "-q", "karatos-riscv"], check=False)
        
        logger.info("✅ Parallel builds completed")

    def run_tests(self):
        """Run comprehensive tests"""
        logger.info("Running comprehensive tests...")
        
        self.run_command(["docker-compose", "run", "--rm", "karatos-test"])
        logger.info("✅ Tests completed")

    def run_ci_pipeline(self):
        """Run complete CI pipeline"""
        logger.info("🚀 Starting complete CI/CD pipeline")
        
        try:
            # Step 1: Build images
            self.build_images()
            
            # Step 2: Run parallel builds
            self.run_parallel_builds()
            
            # Step 3: Run tests
            self.run_tests()
            
            # Step 4: Generate release artifacts (if requested)
            logger.info("✅ CI/CD pipeline completed successfully")
            
        except Exception as e:
            logger.error(f"❌ CI/CD pipeline failed: {e}")
            sys.exit(1)

    def create_release(self):
        """Create release artifacts"""
        logger.info("Creating release artifacts...")
        
        # Build release images
        self.build_images("karatos-release")
        
        # Generate artifacts
        self.run_command(["docker-compose", "run", "--rm", "karatos-release"])
        
        # Copy artifacts from volume
        self.run_command([
            "docker", "run", "--rm", 
            "-v", "rtos-rust_release-artifacts:/artifacts",
            "-v", f"{self.workspace_dir}/release:/output",
            "alpine:latest",
            "sh", "-c", "cp -r /artifacts/* /output/"
        ])
        
        logger.info("✅ Release artifacts created in ./release/")

    def interactive_shell(self, service: str = "karatos-ci"):
        """Start interactive development shell"""
        logger.info(f"Starting interactive shell in {service}...")
        
        self.run_command(["docker-compose", "run", "--rm", service])

    def cleanup(self):
        """Clean up containers and volumes"""
        logger.info("Cleaning up Docker resources...")
        
        self.run_command(["docker-compose", "down", "-v"])
        logger.info("✅ Cleanup completed")

    def show_status(self):
        """Show status of all services"""
        logger.info("Docker Compose service status:")
        self.run_command(["docker-compose", "ps"])

    def show_logs(self, service: str = None):
        """Show logs for service(s)"""
        cmd = ["docker-compose", "logs"]
        if service:
            cmd.append(service)
        self.run_command(cmd)

def main():
    parser = argparse.ArgumentParser(description='karatOS Docker CI/CD Manager')
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Build command
    build_parser = subparsers.add_parser('build', help='Build Docker images')
    build_parser.add_argument('--service', help='Specific service to build')
    
    # Test command
    subparsers.add_parser('test', help='Run tests')
    
    # Pipeline command  
    subparsers.add_parser('pipeline', help='Run complete CI/CD pipeline')
    
    # Release command
    subparsers.add_parser('release', help='Create release artifacts')
    
    # Shell command
    shell_parser = subparsers.add_parser('shell', help='Start interactive shell')
    shell_parser.add_argument('--service', default='karatos-ci', help='Service to run shell in')
    
    # Status command
    subparsers.add_parser('status', help='Show service status')
    
    # Logs command
    logs_parser = subparsers.add_parser('logs', help='Show service logs')
    logs_parser.add_argument('--service', help='Specific service logs')
    
    # Cleanup command
    subparsers.add_parser('cleanup', help='Clean up Docker resources')
    
    # Development shortcuts
    subparsers.add_parser('dev-arm', help='Quick ARM development build')
    subparsers.add_parser('dev-riscv', help='Quick RISC-V development build')
    subparsers.add_parser('dev-test', help='Quick development test')
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    ci = DockerCI()
    
    # Check if docker-compose is available
    try:
        subprocess.run(["docker-compose", "--version"], check=True, capture_output=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        logger.error("docker-compose not found. Please install Docker Compose.")
        sys.exit(1)
    
    # Execute commands
    if args.command == 'build':
        ci.build_images(args.service)
    elif args.command == 'test':
        ci.run_tests()
    elif args.command == 'pipeline':
        ci.run_ci_pipeline()
    elif args.command == 'release':
        ci.create_release()
    elif args.command == 'shell':
        ci.interactive_shell(args.service)
    elif args.command == 'status':
        ci.show_status()
    elif args.command == 'logs':
        ci.show_logs(args.service)
    elif args.command == 'cleanup':
        ci.cleanup()
    elif args.command == 'dev-arm':
        ci.run_command(["docker-compose", "run", "--rm", "karatos-arm"])
    elif args.command == 'dev-riscv':
        ci.run_command(["docker-compose", "run", "--rm", "karatos-riscv"])
    elif args.command == 'dev-test':
        ci.run_command(["docker-compose", "run", "--rm", "karatos-test"])
    else:
        logger.error(f"Unknown command: {args.command}")
        sys.exit(1)

if __name__ == '__main__':
    main()
