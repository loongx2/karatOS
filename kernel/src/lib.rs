//! karatOS Kernel Library
//! Multi-architecture RTOS kernel for ARM and RISC-V platforms

#![no_std]
#![no_main]

// Core modules
pub mod arch;
pub mod config;
pub mod drivers;
pub mod kernel;
pub mod memory;