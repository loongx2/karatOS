//! Build Script for Multi-Architecture Kernel
//! Handles platform-specific build configuration

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    
    // Configure linker script based on target architecture
    if target.starts_with("riscv32") {
        configure_riscv_build(out);
    } else if target.starts_with("arm") || target.starts_with("thumb") {
        configure_arm_build(out);
    } else {
        panic!("Unsupported target architecture: {}", target);
    }
    
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=build.rs");
}

fn configure_riscv_build(out: &PathBuf) {
    // Set RISC-V specific configuration
    println!("cargo:rustc-cfg=riscv_target");
    
    // Use RISC-V specific linker script
    let riscv_linker_script = std::fs::read("../memory-riscv-minimal.x")
        .or_else(|_| std::fs::read("../memory-riscv-simple.x"))
        .or_else(|_| std::fs::read("memory-riscv.x"))
        .expect("Failed to read RISC-V linker script");
        
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(&riscv_linker_script)
        .unwrap();
    
    println!("cargo:rerun-if-changed=../memory-riscv-minimal.x");
    println!("cargo:rerun-if-changed=../memory-riscv-simple.x");
    println!("cargo:rerun-if-changed=memory-riscv.x");
}

fn configure_arm_build(out: &PathBuf) {
    // Set ARM specific configuration
    println!("cargo:rustc-cfg=arm_target");
    
    // Use ARM specific linker script
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    
    println!("cargo:rerun-if-changed=memory.x");
}
