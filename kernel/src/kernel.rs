//! ============================================================================
//! MODULE : kernel — architecture-agnostic kernel bring-up
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   Owns the canonical boot order. Everything above arch/ plugs together
//!   here — this is the single place that knows the startup sequence.
//!
//! BOOT ORDER (Phase 0/1)
//!   1. init_board()            board hooks (clock/power; stub today)
//!   2. arch init (ArchInit)    per-CPU services, irq/memory-protection stubs
//!   3. drivers::init_platform_devices()
//!                              UART first (console live), then timer;
//!                              each driver's init() runs inside the registry
//!   4. arch::init_tick()       SysTick / CLINT @ 1 kHz — time starts flowing
//!   5. boot banner             board + driver status through the registry
//!
//! AFTER init()
//!   The scheduler test loop in main.rs calls arch::advance_time() once per
//!   cycle and feeds scheduler::update_global_timer() — the tick ISR only
//!   ever bumps a latch, keeping ISR latency constant (see arch/mod.rs).
//!
//! OOP MODEL
//!   Pure orchestration: consumes the `ArchInit` trait implementations and
//!   the `Driver` registry; owns no hardware knowledge itself.
//!
//! MEMORY BUDGET
//!   Code only; all state lives in the drivers/registry/arch modules.
//! ============================================================================

use crate::board;
use crate::dtb;
use crate::drivers;
// Trait import is only needed when the arch init calls below compile.
#[cfg(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64"))]
use crate::arch::ArchInit;

/// Initialize the kernel for the current architecture.
///
/// Idempotent-in-intent: called once from the arch entry point before the
/// scheduler loop starts. On host builds every arch-dependent step compiles
/// to a no-op, so host unit tests can call it safely too.
pub fn init(dtb_addr: Option<usize>) {
    // 0. Runtime device tree (Phase 2): probe the firmware-supplied blob.
    //    Boards without one (ARM LM3S under QEMU) fall back to the built-in
    //    table in board.rs — the "OTP/flash config with safe fallback" model.
    let fdt = dtb_addr.and_then(dtb::Fdt::probe);

    // 1. Board bring-up (clocks/power hooks — stub today).
    board::init_board();

    // 2. Architecture services (irq + memory-protection stubs for now).
    #[cfg(target_arch = "arm")]
    crate::arch::arm::ArmArch::init();

    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    crate::arch::riscv::RiscvArch::init();

    // 3. Driver registry from the runtime device list (DTB) or the built-in
    //    table: UART first so the console is live, then the timer.
    let drivers_up = drivers::init_platform_devices(fdt.as_ref().map(|f| f.device_configs()).as_deref());

    // 4. Start the 1 kHz tick (SysTick / CLINT) — time begins here.
    crate::arch::init_tick();

    // 5. Boot banner through the registered console driver (proves the
    //    registry + UART + init order all work end to end).
    let device_count = board::board_config().devices.len();
    let dtb_note = match &fdt {
        Some(f) => {
            let mut s = heapless::String::<48>::new();
            let _ = core::fmt::Write::write_fmt(
                &mut s,
                format_args!(" | dtb: {} B", f.total_size),
            );
            s
        }
        None => heapless::String::new(),
    };
    drivers::console_print("\r\n");
    drivers::console_print("karatOS kernel initialized\r\n");
    drivers::console_print(board::board_name());
    drivers::console_print(dtb_note.as_str());
    drivers::console_print(" | drivers up: ");
    print_usb_dec(drivers_up as u64);
    drivers::console_print(" | devices: ");
    print_usb_dec(device_count as u64);
    drivers::console_print(" | tick: ");
    print_usb_dec(crate::arch::TICK_RATE_HZ as u64);
    drivers::console_print(" Hz\r\n");
}

/// Print a decimal number through the console without any allocation.
fn print_usb_dec(mut value: u64) {
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    if value == 0 {
        drivers::console_print("0");
        return;
    }
    while value > 0 && i > 0 {
        i -= 1;
        buf[i] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    // SAFETY-free: buf[i..] is fully initialized by the loop above.
    if let Ok(s) = core::str::from_utf8(&buf[i..]) {
        drivers::console_print(s);
    }
}

/// Main kernel loop (parked idle path).
///
/// The live scheduler demo in main.rs runs its own loop; this idle loop is
/// the production shape: drain time, run a scheduler cycle, sleep until the
/// next tick.
#[allow(dead_code)]
pub fn run() -> ! {
    drivers::console_print("Kernel running...\r\n");

    loop {
        let _now = crate::arch::advance_time();
        // Scheduling cycle hook goes here once the executor moves into the
        // kernel proper (Phase 3).

        #[cfg(any(target_arch = "arm", target_arch = "riscv32", target_arch = "riscv64"))]
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
