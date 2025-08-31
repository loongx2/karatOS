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
        // For host targets (x86_64, etc.) used in testing, do nothing
        // This allows cargo test to work on development machines
        println!("cargo:rustc-cfg=host_target");
        return;
    }
    
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=build.rs");
}

fn configure_riscv_build(out: &PathBuf) {
    // Set RISC-V specific configuration
    println!("cargo:rustc-cfg=riscv_target");

    // Use RISC-V specific linker script from templates
    let template_path = std::env::var("CARGO_MANIFEST_DIR")
        .map(|dir| PathBuf::from(dir).join("../build/templates/memory-riscv.x"))
        .unwrap_or_else(|_| PathBuf::from("../build/templates/memory-riscv.x"));

    let riscv_linker_script = std::fs::read(&template_path)
        .unwrap_or_else(|_| {
            // Fallback to kernel directory if template not found
            std::fs::read("memory-riscv.x")
                .expect("Failed to read RISC-V linker script memory-riscv.x from kernel/ or ../build/templates/")
        });

    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(&riscv_linker_script)
        .unwrap();

    println!("cargo:rerun-if-changed=memory-riscv.x");
    println!("cargo:rerun-if-changed=../build/templates/memory-riscv.x");
}

fn configure_arm_build(out: &PathBuf) {
    // Set ARM specific configuration
    println!("cargo:rustc-cfg=arm_target");

    // Use ARM specific linker script from templates
    let template_path = std::env::var("CARGO_MANIFEST_DIR")
        .map(|dir| PathBuf::from(dir).join("../build/templates/memory-arm.x"))
        .unwrap_or_else(|_| PathBuf::from("../build/templates/memory-arm.x"));

    let arm_linker_script = std::fs::read(&template_path)
        .unwrap_or_else(|_| {
            // Fallback to kernel directory if template not found
            std::fs::read("memory-arm.x")
                .expect("Failed to read ARM linker script memory-arm.x from kernel/ or ../build/templates/")
        });

    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(&arm_linker_script)
        .unwrap();

    println!("cargo:rerun-if-changed=memory-arm.x");
    println!("cargo:rerun-if-changed=../build/templates/memory-arm.x");
}