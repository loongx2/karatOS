//! ============================================================================
//! MODULE : drivers::uart_simple — console UART drivers (PL011 / NS16550A)
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   Implements the `Driver` contract for the two console UARTs karatOS
//!   supports today:
//!     * `Pl011Uart`    — ARM PL011 (LM3S6965EVB UART0 @ 0x4000_C000)
//!     * `Ns16550Uart`  — RISC-V NS16550A (QEMU virt UART0 @ 0x1000_0000)
//!
//! ROLE IN BOOT FLOW
//!   Instantiated as `PLATFORM_UART` statics in drivers/mod.rs, registered
//!   FIRST so the console is live before any other driver inits. Early boot
//!   output (before registration) still goes through `arch::early_println`.
//!
//! MEMORY BUDGET
//!   One `usize` (base address) per instance. Register constants live in
//!   .rodata / are folded into immediates by LTO.
//!
//! OOP MODEL
//!   Both types implement `Driver`; the console capability methods
//!   (`write_byte`/`write_str`) override the trait's no-op defaults.
//!
//! NOTE ON QEMU
//!   QEMU models both UARTs but ignores baud divisors, so init keeps the
//!   divisor programming minimal but correct for real silicon.
//! ============================================================================

use super::{DeviceClass, Driver, DriverError, VolatileReg};

// ---------------------------------------------------------------------------
// ARM PL011 (primecell UART)
// ---------------------------------------------------------------------------
/// Register offsets from the PL011 base address (ARM DDI 0183).
mod pl011 {
    pub const DR: usize = 0x000; // Data register
    pub const FR: usize = 0x018; // Flag register
    pub const IBRD: usize = 0x024; // Integer baud rate divisor
    pub const FBRD: usize = 0x028; // Fractional baud rate divisor
    pub const LCR_H: usize = 0x02C; // Line control
    pub const CTL: usize = 0x030; // Control

    pub const FR_TXFF: u32 = 1 << 5; // TX FIFO full
    pub const LCR_FEN: u32 = 1 << 4; // FIFO enable
    pub const CTL_UARTEN: u32 = 1 << 0; // UART enable
    pub const CTL_TXE: u32 = 1 << 8; // TX enable
}

/// PL011 console driver (8N1, TX use).
pub struct Pl011Uart {
    base: usize,
}

impl Pl011Uart {
    /// LM3S6965EVB UART0 base address.
    pub const BASE: usize = 0x4000_C000;

    /// Const constructor for the platform singleton (default base).
    pub const fn new() -> Self {
        Self { base: Self::BASE }
    }

    /// Const constructor at a runtime-discovered base (DTB Phase 2).
    pub const fn at_base(base: usize) -> Self {
        Self { base }
    }

    #[inline(always)]
    fn reg(&self, offset: usize) -> VolatileReg {
        // SAFETY: base is a board/DTB-verified MMIO address.
        unsafe { VolatileReg::new(self.base + offset) }
    }
}

impl Default for Pl011Uart {
    fn default() -> Self {
        Self::new()
    }
}

impl Driver for Pl011Uart {
    fn name(&self) -> &'static str {
        "uart0"
    }

    fn class(&self) -> DeviceClass {
        DeviceClass::Uart
    }

    /// Enable the UART0 clock gate, program 115200 8N1, then enable UART+TX.
    fn init(&mut self) -> Result<(), DriverError> {
        const RCGC1: usize = 0x400F_E104; // Run-mode clock gating control 1
        const UART0_CLK_GATE: u32 = 1 << 0;

        // SAFETY: fixed LM3S6965 system-control MMIO address.
        unsafe {
            let rcgc1 = VolatileReg::new(RCGC1);
            rcgc1.set_bits32(UART0_CLK_GATE);
        }

        // Divisors for 115200 baud from a 12 MHz clock:
        //   IBRD = 12e6 / (16 * 115200) = 6
        //   FBRD = round(0.5104 * 64)   = 33
        self.reg(pl011::CTL).write32(0); // disable while reconfiguring
        self.reg(pl011::IBRD).write32(6);
        self.reg(pl011::FBRD).write32(33);
        // 8-bit words (WLEN=0b11), FIFOs on
        self.reg(pl011::LCR_H).write32(pl011::LCR_FEN | (0x3 << 5));
        self.reg(pl011::CTL)
            .write32(pl011::CTL_UARTEN | pl011::CTL_TXE);
        Ok(())
    }

    /// Blocking transmit of one byte.
    fn write_byte(&mut self, byte: u8) {
        // Wait while the TX FIFO is full, then push the byte.
        while (self.reg(pl011::FR).read32() & pl011::FR_TXFF) != 0 {
            core::hint::spin_loop();
        }
        self.reg(pl011::DR).write32(byte as u32);
    }
}

// ---------------------------------------------------------------------------
// RISC-V NS16550A (QEMU virt UART0)
// ---------------------------------------------------------------------------
/// Register offsets from the NS16550 base address (byte-wide registers).
mod ns16550 {
    pub const THR: usize = 0x00; // Transmit holding register (write)
    pub const RBR: usize = 0x00; // Receive buffer register (read)
    pub const IER: usize = 0x01; // Interrupt enable
    pub const FCR: usize = 0x02; // FIFO control
    pub const LCR: usize = 0x03; // Line control
    pub const LSR: usize = 0x05; // Line status register

    pub const LSR_THRE: u8 = 1 << 5; // TX holding register empty
    pub const LSR_DR: u8 = 1 << 0; // Data ready
}

/// NS16550A console driver (8N1, TX use; RX helper included).
pub struct Ns16550Uart {
    base: usize,
}

impl Ns16550Uart {
    /// QEMU virt UART0 base address.
    pub const BASE: usize = 0x1000_0000;

    /// Const constructor for the platform singleton (default base).
    pub const fn new() -> Self {
        Self { base: Self::BASE }
    }

    /// Const constructor at a runtime-discovered base (DTB Phase 2).
    pub const fn at_base(base: usize) -> Self {
        Self { base }
    }

    #[inline(always)]
    fn reg(&self, offset: usize) -> VolatileReg {
        // SAFETY: base is a machine/DTB-verified MMIO address.
        unsafe { VolatileReg::new(self.base + offset) }
    }
}

impl Default for Ns16550Uart {
    fn default() -> Self {
        Self::new()
    }
}

impl Driver for Ns16550Uart {
    fn name(&self) -> &'static str {
        "uart0"
    }

    fn class(&self) -> DeviceClass {
        DeviceClass::Uart
    }

    /// 8N1, FIFO on, polling mode (UART interrupts disabled).
    /// Divisor programming is skipped: QEMU's model has no timing, and on
    /// real silicon the boot ROM has usually already programmed the divisor.
    fn init(&mut self) -> Result<(), DriverError> {
        self.reg(ns16550::IER).write8(0x00); // polled mode
        self.reg(ns16550::LCR).write8(0x03); // 8 data bits, 1 stop, no parity
        self.reg(ns16550::FCR).write8(0x07); // enable + clear FIFOs
        Ok(())
    }

    /// Blocking transmit of one byte.
    fn write_byte(&mut self, byte: u8) {
        // Wait while the TX holding register is not empty, then transmit.
        while (self.reg(ns16550::LSR).read8() & ns16550::LSR_THRE) == 0 {
            core::hint::spin_loop();
        }
        self.reg(ns16550::THR).write8(byte);
    }
}

impl Ns16550Uart {
    /// Non-blocking receive: `Some(byte)` when data is ready.
    #[allow(dead_code)] // RX surface kept for Phase 2 console input
    pub fn read_byte(&mut self) -> Option<u8> {
        let lsr = self.reg(ns16550::LSR).read8();
        if (lsr & ns16550::LSR_DR) != 0 {
            Some(self.reg(ns16550::RBR).read8())
        } else {
            None
        }
    }
}
