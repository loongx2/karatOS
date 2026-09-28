//! ============================================================================
//! MODULE : board — board description layer (what hardware does this SoC have?)
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   Names the board and lists its on-chip devices as `DeviceConfig` blobs.
//!   This is the compile-time stand-in for the Phase-2 runtime device tree:
//!   today the table is a `const` selected by target architecture; tomorrow
//!   the same table will be parsed from a DTB image stored in flash/OTP and
//!   selected by a board ID burned into OTP. Boot code shape stays identical.
//!
//! ROLE IN BOOT FLOW
//!   kernel::init() -> init_board() (clock/power hooks) -> device_configs()
//!   consumed by drivers::init_platform_devices().
//!
//! MEMORY BUDGET
//!   Pure .rodata (const tables); no runtime RAM cost.
//!
//! OOP MODEL
//!   Data only — the *behavior* lives behind the `Driver` trait; the board
//!   layer just pairs device instances with driver singletons by name.
//! ============================================================================

use crate::drivers::DeviceConfig;
// `DeviceClass` is only referenced by the per-target device tables below;
// the host table is empty, so the import would be unused there.
#[cfg(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64"))]
use crate::drivers::{DeviceClass, DriverFlavor};

// Core-clock constants come from the arch layer (single source of truth for
// tick math and board tables alike).
#[cfg(target_arch = "arm")]
use crate::arch::arm::CORE_CLOCK_HZ;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
use crate::arch::riscv::CORE_CLOCK_HZ;

/// This board's OTP-equivalent identity (karatos_kapi::board_id values).
/// On real silicon this constant is REPLACED at boot by a read of the OTP
/// block; in QEMU the board is known by construction.
// Per-target OTP-equivalent board identity. Separate `const` items with
// exclusive `#[cfg]`s (instead of one cfg-picked block) stay total on every
// target, including host analysis where no branch matches.
#[cfg(all(target_arch = "arm", not(armv8m_target)))]
pub const BOARD_ID: u16 = karatos_kapi::board_id::LM3S6965;
#[cfg(all(target_arch = "arm", armv8m_target))]
pub const BOARD_ID: u16 = karatos_kapi::board_id::MPS3_AN547;
#[cfg(all(target_arch = "riscv32", imc_target))]
pub const BOARD_ID: u16 = karatos_kapi::board_id::QEMU_VIRT_RV32IMC;
#[cfg(all(target_arch = "riscv32", not(imc_target)))]
pub const BOARD_ID: u16 = karatos_kapi::board_id::QEMU_VIRT_RV32IMAC;
#[cfg(target_arch = "riscv64")]
pub const BOARD_ID: u16 = karatos_kapi::board_id::QEMU_VIRT_RV64;
#[cfg(not(any(
    target_arch = "arm",
    target_arch = "riscv32",
    target_arch = "riscv64"
)))]
pub const BOARD_ID: u16 = karatos_kapi::board_id::UNPROVISIONED;


/// One board's static description.
#[allow(dead_code)] // host build never enumerates devices; DTB replaces in Phase 2
pub struct BoardConfig {
    /// Human-readable board name (printed on the boot banner).
    pub board_name: &'static str,
    /// On-chip devices, in registration order (UART first!).
    pub devices: &'static [DeviceConfig],
}

// ---------------------------------------------------------------------------
// Per-architecture device tables
// ---------------------------------------------------------------------------

// LM3S6965EVB: PL011 UART0 + core SysTick (base = SysTick control register).
#[cfg(target_arch = "arm")]
pub static DEVICES: &[DeviceConfig] = &[
    DeviceConfig {
        name: "uart0",
        class: DeviceClass::Uart,
        flavor: DriverFlavor::Pl011,
        base: 0x4000_C000,
        clock_hz: CORE_CLOCK_HZ,
        baud: Some(115_200),
    },
    DeviceConfig {
        name: "timer0",
        class: DeviceClass::Timer,
        flavor: DriverFlavor::SysTick,
        base: 0xE000_E010, // SysTick CSR — informational, core-private
        clock_hz: CORE_CLOCK_HZ,
        baud: None,
    },
];

// QEMU virt (RV32/RV64): NS16550A UART0 + CLINT timer (base = CLINT mtimecmp).
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub static DEVICES: &[DeviceConfig] = &[
    DeviceConfig {
        name: "uart0",
        class: DeviceClass::Uart,
        flavor: DriverFlavor::Ns16550,
        base: 0x1000_0000,
        clock_hz: 0, // NS16550 is clock-less under QEMU
        baud: Some(115_200),
    },
    DeviceConfig {
        name: "timer0",
        class: DeviceClass::Timer,
        flavor: DriverFlavor::Clint,
        base: 0x0200_4000, // CLINT mtimecmp for hart 0
        clock_hz: CORE_CLOCK_HZ,
        baud: None,
    },
];

// Host build: no real devices.
#[cfg(not(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64")))]
pub static DEVICES: &[DeviceConfig] = &[];

/// Board name for the current target.
pub const fn board_name() -> &'static str {
    #[cfg(target_arch = "arm")]
    {
        "LM3S6965EVB (Cortex-M3)"
    }
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    {
        "QEMU virt (RISC-V)"
    }
    #[cfg(not(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64")))]
    {
        "Host (no board)"
    }
}

/// Full board configuration for the current target.
#[allow(dead_code)] // host build: empty device table, config unused
pub const fn board_config() -> BoardConfig {
    BoardConfig {
        board_name: board_name(),
        devices: DEVICES,
    }
}

/// Board early bring-up hook (clocks, power domains).
///
/// Real clock gating happens inside the drivers that need it (e.g. the PL011
/// driver gates its own clock), so this stays a stub today. In Phase 2/5 this
/// is where DTB-declared clock/pinmux setup will run.
pub fn init_board() {
    // No-op: see module docs.
}
