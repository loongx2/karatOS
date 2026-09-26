//! Build Script for Multi-Architecture Kernel
//! Handles platform-specific build configuration

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Resolve the memory layout template for the current target.
///
/// Processor selection is fully externalized here so that adding a new core
/// never requires touching kernel code (OCP: extend this table only).
///   1. `KARATOS_MEMORY_TEMPLATE` - explicit linker script file name/path
///   2. `KARATOS_CORE`            - named core variant, mapped below
///   3. default per-architecture-family template
fn resolve_memory_template(target: &str) -> PathBuf {
    let templates_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../build/templates");

    // Explicit override wins
    if let Ok(path) = env::var("KARATOS_MEMORY_TEMPLATE") {
        let p = PathBuf::from(&path);
        return if p.exists() { p } else { templates_dir.join(path) };
    }

    // Named core variants (new cores are registered HERE only)
    let core_variant = env::var("KARATOS_CORE").unwrap_or_default();
    let name: &str = match core_variant.as_str() {
        "mps3-an547" | "musca-b1" | "cortex-m33" | "cortex-m55" => "memory-arm-v8m.x",
        "lm3s6965" | "cortex-m3" => "memory-arm.x",
        "virt-rv32imc" => "memory-riscv-imc.x",
        "virt-rv64gc" => "memory-riscv64.x",
        "virt-rv32imac" | "" => {
            if target.starts_with("riscv64") {
                "memory-riscv64.x"
            } else if target.starts_with("riscv32imc-") {
                "memory-riscv-imc.x"
            } else if target.starts_with("riscv") {
                "memory-riscv.x"
            } else {
                "memory-arm.x"
            }
        }
        other => panic!("karatOS: unknown KARATOS_CORE '{}' (register it in kernel/build.rs)", other),
    };

    templates_dir.join(name)
}

fn main() {
    let target = env::var("TARGET").unwrap();
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // Re-run when the selected core changes so memory.x always matches
    println!("cargo:rerun-if-env-changed=KARATOS_CORE");
    println!("cargo:rerun-if-env-changed=KARATOS_MEMORY_TEMPLATE");
    println!("cargo:rerun-if-changed=../build/templates");

    // Configure linker script based on target architecture
    if target.starts_with("riscv") {
        configure_riscv_build(out, &target);
    } else if target.starts_with("arm") || target.starts_with("thumb") {
        configure_arm_build(out, &target);
    } else {
        // For host targets (x86_64, etc.) used in testing, do nothing
        // This allows cargo test to work on development machines
        println!("cargo:rustc-cfg=host_target");
        return;
    }
    
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=build.rs");
}

fn configure_riscv_build(out: &PathBuf, target: &str) {
    // Set RISC-V specific configuration
    println!("cargo:rustc-cfg=riscv_target");
    if target.starts_with("riscv64") {
        println!("cargo:rustc-cfg=riscv64_target");
    }

    let template_path = resolve_memory_template(target);

    let riscv_linker_script = std::fs::read(&template_path)
        .unwrap_or_else(|e| {
            panic!(
                "karatOS: failed to read RISC-V linker script {} ({})",
                template_path.display(),
                e
            )
        });

    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(&riscv_linker_script)
        .unwrap();

    println!("cargo:rerun-if-changed={}", template_path.display());
}

fn configure_arm_build(out: &PathBuf, target: &str) {
    // Set ARM specific configuration
    println!("cargo:rustc-cfg=arm_target");
    if target.starts_with("thumbv8") {
        println!("cargo:rustc-cfg=armv8m_target");
    }

    let template_path = resolve_memory_template(target);

    let arm_linker_script = std::fs::read(&template_path)
        .unwrap_or_else(|e| {
            panic!(
                "karatOS: failed to read ARM linker script {} ({})",
                template_path.display(),
                e
            )
        });

    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(&arm_linker_script)
        .unwrap();

    println!("cargo:rerun-if-changed={}", template_path.display());
}