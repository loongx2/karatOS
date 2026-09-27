//! ============================================================================
//! karatOS Kernel Library (`kernel_lib`)
//! ----------------------------------------------------------------------------
//! Multi-architecture RTOS kernel for ARM (Cortex-M) and RISC-V (RV32/RV64),
//! built around three contracts:
//!
//!   1. `arch`      — the only layer with `target_arch` knowledge (ARM ⇄ RISC-V)
//!   2. `drivers`   — the only layer that touches MMIO (`Driver` trait +
//!                    `DriverRegistry`; the plug-point for future modules)
//!   3. `scheduler` — cooperative priority executor fed REAL time from the
//!                    1 kHz arch tick (see `arch::advance_time`)
//!
//! `board` names the silicon and lists its devices; `kernel::init` performs
//! the canonical boot order; `config`/`memory` describe the target shape.
//!
//! LIBRARY vs BINARY
//!   This lib (and its `cargo test --lib` host unit tests) shares sources
//!   with the `kernel` binary, which adds `main.rs` (entries + demo loop).
//!   The lib is `no_std` everywhere; on host builds all arch steps are
//!   compiled out, so registry/scheduler logic is testable off-target.
//! ============================================================================

// Tests run on the host with std linked in (for diagnostics like eprintln!
// in the dtb walker); all non-test builds stay strictly no_std.
#![cfg_attr(not(test), no_std)]
#![cfg(test)]
#[cfg(test)]
extern crate std;

// Core modules
pub mod arch;
pub mod dtb;
pub mod kapi;
pub mod modules;
pub mod board;
pub mod config;
pub mod drivers;
pub mod kernel;
pub mod logger;
pub mod memory;
pub mod scheduler;
pub mod store;