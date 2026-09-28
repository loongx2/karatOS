//! ============================================================================
//! MODULE : drivers — karatOS Hardware Abstraction Layer (HAL) / OOP core
//! ----------------------------------------------------------------------------
//! PURPOSE
//!   Defines the single seam through which ALL hardware reaches the kernel:
//!   the `Driver` trait, per-device `DeviceConfig` blobs, and a fixed-size
//!   `DriverRegistry` of `dyn Driver` trait objects (vtable polymorphism,
//!   zero heap). This is the plug-point where flash-loaded modules (Phase 3)
//!   will register themselves at runtime.
//!
//! ROLE IN BOOT FLOW
//!   kernel::init() -> init_platform_devices() -> DriverRegistry::register()
//!   per device -> init_all() -> drivers live.
//!
//! MEMORY BUDGET
//!   Registry = MAX_DRIVERS * (fat pointer + Option tag) + count, all in
//!   .bss. Drivers themselves are `static mut` singletons; the registry only
//!   stores `*mut dyn Driver` handles, never copies.
//!
//! OOP MODEL
//!   `Driver`       — lifecycle contract (name/class/init/deinit).
//!   Capability fns — `write_byte` (console) and `read_ticks` (timer) live on
//!                    the same trait with default no-op bodies. Deliberate
//!                    64 kB-budget decision: ONE vtable per driver, no
//!                    downcasting machinery, no RTTI.
//!   MMIO           — `VolatileReg` is the ONLY sanctioned way to touch
//!                    device registers from driver code.
//!
//! CONCURRENCY CONTRACT
//!   Single-core, single-threaded boot/registration (kernel::init runs before
//!   the scheduler starts). The global registry is guarded by the same
//!   interrupt-disable critical section used by the scheduler.
//! ============================================================================

use core::cell::UnsafeCell;

// Platform driver implementations (UART + timer behind the `Driver` trait)
pub mod timer;
pub mod uart_simple;

// ---------------------------------------------------------------------------
// Device model
// ---------------------------------------------------------------------------

/// Upper bound on simultaneously registered drivers (fixed allocation).
pub const MAX_DRIVERS: usize = 8;

/// Errors returned by the driver framework.
#[allow(dead_code)] // full error surface is part of the stable module ABI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    /// No implementation for this device type on the current target.
    UnsupportedType,
    /// Hardware did not come up as expected.
    InitFailed,
    /// Requested driver is not in the registry.
    NotFound,
    /// Registry full — no free slots.
    Busy,
}

/// Functional class of a device, used for capability dispatch.
#[allow(dead_code)] // capability dispatch arrives with scheduler integration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    /// Character console (PL011, NS16550A, ...).
    Uart,
    /// Monotonic time source (SysTick, CLINT, ...).
    Timer,
}

impl DeviceClass {
    /// Class a driver flavor serves (DTB layer maps flavors to instances).
    pub fn from_flavor(flavor: DriverFlavor) -> DeviceClass {
        match flavor {
            DriverFlavor::Pl011 | DriverFlavor::Ns16550 => DeviceClass::Uart,
            DriverFlavor::SysTick | DriverFlavor::Clint => DeviceClass::Timer,
        }
    }
}

/// Concrete driver implementation a `compatible` string selects. Unlike the
/// class, this picks the actual MMIO register layout to drive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverFlavor {
    /// ARM PrimeCell PL011 register layout.
    Pl011,
    /// NS16550 register layout (QEMU virt, ESP32-C3, SiFive, ...).
    Ns16550,
    /// ARM core SysTick (built-in fallback; not in DTBs).
    SysTick,
    /// RISC-V CLINT mtime (feeds the tick on RISC-V targets).
    Clint,
}

/// One device instance description. In Phase 2 these blobs will be filled
/// from the flat device tree (DTB) read out of flash/OTP instead of being
/// compile-time constants — the struct is already shaped for that.
#[allow(dead_code)] // consumed by board tables today; DTB parser lands in Phase 2
#[derive(Debug, Clone, Copy)]
pub struct DeviceConfig {
    /// Registry-visible name (stable, lower-case, no whitespace).
    pub name: &'static str,
    /// Functional class (capability dispatch; host demo never reads it).
    #[allow(dead_code)]
    pub class: DeviceClass,
    /// Concrete driver implementation to instantiate.
    pub flavor: DriverFlavor,
    /// MMIO base address.
    pub base: usize,
    /// Input clock feeding the device (Hz).
    pub clock_hz: u32,
    /// Requested baud rate, if applicable.
    pub baud: Option<u32>,
}

// ---------------------------------------------------------------------------
// Driver trait — the kernel's only hardware contract
// ---------------------------------------------------------------------------

/// Lifecycle + capability contract every karatOS driver implements.
///
/// The capability methods use default no-op bodies so that one vtable serves
/// every driver class. Callers must only invoke the capabilities matching the
/// driver's `class()`.
pub trait Driver: 'static {
    /// Stable registry name.
    fn name(&self) -> &'static str;
    /// Functional class for capability dispatch.
    #[allow(dead_code)] // consumed when capability dispatch is wired up
    fn class(&self) -> DeviceClass;
    /// Bring the hardware up. Called once, during kernel boot.
    fn init(&mut self) -> Result<(), DriverError>;
    /// Quiesce the hardware. Default: nothing (retains state).
    #[allow(dead_code)] // module unload (Phase 3) is the first caller
    fn deinit(&mut self) {}

    // -- Capability: console output (DeviceClass::Uart) ----------------------
    /// Blocking single-byte transmit. No-op unless this is a Uart driver.
    fn write_byte(&mut self, _byte: u8) {}
    /// Line-oriented convenience built on `write_byte`.
    fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_byte(b);
        }
    }

    // -- Capability: monotonic time (DeviceClass::Timer) ---------------------
    /// Milliseconds since boot (fed by the architecture tick ISR).
    /// Returns 0 unless this is a Timer driver.
    #[allow(dead_code)] // timer capability; consumed by sleep/wake integration
    fn read_ticks(&self) -> u64 {
        0
    }
}

// ---------------------------------------------------------------------------
// Volatile MMIO access — the only sanctioned register access path
// ---------------------------------------------------------------------------

/// A single memory-mapped register. All reads/writes go through volatile
/// accesses; construction is `unsafe` because the caller asserts the address
/// is a valid device register on this target.
#[repr(transparent)]
pub struct VolatileReg {
    addr: usize,
}

impl VolatileReg {
    /// Wrap a register address. SAFETY: `addr` must be a real MMIO register.
    #[inline(always)]
    pub const unsafe fn new(addr: usize) -> Self {
        Self { addr }
    }

    /// 32-bit volatile read.
    #[inline(always)]
    pub fn read32(&self) -> u32 {
        unsafe { core::ptr::read_volatile(self.addr as *const u32) }
    }

    /// 32-bit volatile write.
    #[inline(always)]
    pub fn write32(&self, value: u32) {
        unsafe { core::ptr::write_volatile(self.addr as *mut u32, value) }
    }

    /// 8-bit volatile read.
    #[inline(always)]
    pub fn read8(&self) -> u8 {
        unsafe { core::ptr::read_volatile(self.addr as *const u8) }
    }

    /// 8-bit volatile write.
    #[inline(always)]
    pub fn write8(&self, value: u8) {
        unsafe { core::ptr::write_volatile(self.addr as *mut u8, value) }
    }

    /// Read-modify-write: OR `bits` into the register.
    #[inline(always)]
    pub fn set_bits32(&self, bits: u32) {
        self.write32(self.read32() | bits);
    }

    /// Read-modify-write: clear `bits` from the register.
    #[allow(dead_code)] // symmetric API; first user is the NVIC/PLIC driver
    #[inline(always)]
    pub fn clear_bits32(&self, bits: u32) {
        self.write32(self.read32() & !bits);
    }
}

// ---------------------------------------------------------------------------
// DriverRegistry — fixed-size table of driver handles
// ---------------------------------------------------------------------------

/// Fixed-capacity registry of `dyn Driver` handles.
///
/// SAFETY CONTRACT (documented once, relied upon everywhere):
///   * drivers are registered exactly once, from `kernel::init()`, before the
///     scheduler starts;
///   * handles are raw `*mut dyn Driver` only so the registry can hand out
///     short-lived `&mut dyn Driver` borrows through a shared static;
///   * no borrow outlives a `find`/`with_slot` call, and every access to the
///     global registry runs inside an interrupt-disabled critical section —
///     therefore no aliasing `&mut` can ever be observed.
pub struct DriverRegistry {
    slots: [Option<*mut dyn Driver>; MAX_DRIVERS],
    count: usize,
}

impl DriverRegistry {
    /// Empty registry (const, lands in .bss).
    pub const fn new() -> Self {
        Self {
            slots: [None; MAX_DRIVERS],
            count: 0,
        }
    }

    /// Number of registered (not necessarily initialized) drivers.
    #[allow(dead_code)] // introspection surface for Phase 3 module manager
    pub fn len(&self) -> usize {
        self.count
    }

    /// Whether no driver has been registered yet.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Register a driver singleton. Registration order == init order.
    pub fn register(&mut self, driver: &'static mut dyn Driver) -> Result<usize, DriverError> {
        if self.count >= MAX_DRIVERS {
            return Err(DriverError::Busy);
        }
        self.slots[self.count] = Some(driver as *mut dyn Driver);
        self.count += 1;
        Ok(self.count - 1)
    }

    /// Borrow driver `idx` mutably for the duration of `f`.
    fn with_slot<F, R>(&mut self, idx: usize, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Driver) -> R,
    {
        let ptr = self.slots[idx]?;
        // SAFETY: pointer originated from a 'static mut reference; per the
        // registry SAFETY CONTRACT no other borrow is live during `f`.
        let driver = unsafe { &mut *ptr };
        Some(f(driver))
    }

    /// Run `init()` on every registered driver, in registration order.
    /// Returns the number of drivers that came up successfully.
    pub fn init_all(&mut self) -> usize {
        let mut ok = 0;
        for idx in 0..self.count {
            let up = self.with_slot(idx, |d| usize::from(d.init().is_ok()));
            ok += up.unwrap_or(0);
        }
        ok
    }

    /// Find a driver by exact name and borrow it mutably for `f`.
    pub fn find<F, R>(&mut self, name: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Driver) -> R,
    {
        for idx in 0..self.count {
            let matched = self.with_slot(idx, |d| d.name() == name);
            if matched == Some(true) {
                return self.with_slot(idx, f);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Global registry (boot-time singletons, scheduler-style critical section)
// ---------------------------------------------------------------------------

struct RegistryCell(UnsafeCell<DriverRegistry>);
// SAFETY: single-core; all access is funnelled through `with_registry`'s
// interrupt-disabled critical section (same discipline as scheduler.rs).
unsafe impl Sync for RegistryCell {}

static REGISTRY: RegistryCell = RegistryCell(UnsafeCell::new(DriverRegistry::new()));

/// Access the global registry inside an interrupt-disabled critical section.
#[inline(always)]
fn with_registry<F, R>(f: F) -> R
where
    F: FnOnce(&mut DriverRegistry) -> R,
{
    crate::arch::disable_interrupts();
    // SAFETY: see RegistryCell Sync impl — critical section serializes access.
    let result = unsafe { f(&mut *REGISTRY.0.get()) };
    crate::arch::enable_interrupts();
    result
}

// ---------------------------------------------------------------------------
// Platform device bring-up — Phase 2: runtime DTB enumeration with a safe
// compile-time fallback. The driver SINGLETONS below exist on every target
// (they are plain MMIO state machines; only init()/write_byte() touch the
// hardware, and only the flavors the device list asks for are registered).
// ---------------------------------------------------------------------------

/// PL011 console singleton (registered when a DTB/board config asks for it).
static mut DRV_PL011: uart_simple::Pl011Uart = uart_simple::Pl011Uart::new();

/// NS16550 console singleton (registered when a DTB/board config asks for it).
static mut DRV_16550: uart_simple::Ns16550Uart = uart_simple::Ns16550Uart::new();

/// Monotonic tick source singleton (fed by the arch tick ISR).
static mut DRV_TIMER: timer::TickTimerDriver = timer::TickTimerDriver::new();

/// Register the platform drivers described by `devices` into the global
/// registry and run `init()` on each. `devices` comes from the DTB walk
/// (Phase 2) when a blob was found, otherwise from the built-in board table.
/// The console is always registered FIRST so later drivers can report.
/// Returns the number of drivers that came up.
pub fn init_platform_devices(devices: Option<&[DeviceConfig]>) -> usize {
    // Runtime list wins; compile-time table is the safe fallback.
    let configs: &[DeviceConfig] = match devices {
        Some(list) if !list.is_empty() => list,
        _ => crate::board::DEVICES,
    };

    // Pass 1: console — guarantees output before any other driver reports.
    for cfg in configs {
        if cfg.class == DeviceClass::Uart {
            let accepted = with_registry(|reg| match cfg.flavor {
                DriverFlavor::Pl011 => {
                    // SAFETY: singleton re-initialized before scheduler start.
                    unsafe {
                        core::ptr::write(core::ptr::addr_of_mut!(DRV_PL011), uart_simple::Pl011Uart::at_base(cfg.base));
                    }
                    reg.register(unsafe { &mut *core::ptr::addr_of_mut!(DRV_PL011) }).is_ok()
                }
                DriverFlavor::Ns16550 => {
                    // SAFETY: singleton re-initialized before scheduler start.
                    unsafe {
                        core::ptr::write(core::ptr::addr_of_mut!(DRV_16550), uart_simple::Ns16550Uart::at_base(cfg.base));
                    }
                    reg.register(unsafe { &mut *core::ptr::addr_of_mut!(DRV_16550) }).is_ok()
                }
                _ => false,
            });
            if accepted {
                break;
            }
        }
    }

    // Pass 2: monotonic timer.
    for cfg in configs {
        if cfg.class == DeviceClass::Timer {
            let accepted = with_registry(|reg| {
                // SAFETY: singleton re-initialized before scheduler start.
                unsafe {
                    core::ptr::write(core::ptr::addr_of_mut!(DRV_TIMER), timer::TickTimerDriver::with_clock(cfg.clock_hz));
                }
                reg.register(unsafe { &mut *core::ptr::addr_of_mut!(DRV_TIMER) }).is_ok()
            });
            if accepted {
                break;
            }
        }
    }

    with_registry(|reg| reg.init_all())
}

/// Kernel console: write a string through the registered UART driver.
/// Silently drops output when the UART is not registered (early boot uses
/// `arch::early_println` instead).
pub fn console_print(msg: &str) {
    with_registry(|reg| {
        let _ = reg.find("uart0", |d| d.write_str(msg));
    });
}

// ---------------------------------------------------------------------------
// Host unit tests — pure registry logic, no MMIO touched
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicU64, Ordering};

    /// Test double: counts init calls and written bytes, returns fixed ticks.
    struct FakeDriver {
        id: &'static str,
        #[allow(dead_code)] // mirrors DeviceConfig; dispatch goes via trait
        class: DeviceClass,
        inits: AtomicU64,
        output: AtomicU64,
    }

    impl FakeDriver {
        const fn new(id: &'static str, class: DeviceClass) -> Self {
            Self {
                id,
                class,
                inits: AtomicU64::new(0),
                output: AtomicU64::new(0),
            }
        }
    }

    impl Driver for FakeDriver {
        fn name(&self) -> &'static str {
            self.id
        }
        fn class(&self) -> DeviceClass {
            self.class
        }
        fn init(&mut self) -> Result<(), DriverError> {
            self.inits.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
        fn write_byte(&mut self, _b: u8) {
            self.output.fetch_add(1, Ordering::Relaxed);
        }
        fn read_ticks(&self) -> u64 {
            42
        }
    }

    static mut FAKE_UART: FakeDriver = FakeDriver::new("uart0", DeviceClass::Uart);
    static mut FAKE_TIMER: FakeDriver = FakeDriver::new("timer0", DeviceClass::Timer);

    #[test]
    fn register_and_init_in_order() {
        let mut reg = DriverRegistry::new();
        assert!(reg.is_empty());
        // SAFETY (test): re-borrowing statics with no live aliasing borrow.
        assert_eq!(reg.register(unsafe { &mut *core::ptr::addr_of_mut!(FAKE_UART) }), Ok(0));
        assert_eq!(reg.register(unsafe { &mut *core::ptr::addr_of_mut!(FAKE_TIMER) }), Ok(1));
        assert_eq!(reg.init_all(), 2);
        assert_eq!(reg.len(), 2);
        assert!(!reg.is_empty());
    }

    #[test]
    fn find_by_name_dispatches_capability() {
        let mut reg = DriverRegistry::new();
        // SAFETY (test): re-borrowing statics with no live aliasing borrow.
        let _ = reg.register(unsafe { &mut *core::ptr::addr_of_mut!(FAKE_UART) });
        let _ = reg.register(unsafe { &mut *core::ptr::addr_of_mut!(FAKE_TIMER) });

        let name_len = reg.find("uart0", |d| {
            d.write_str("hello");
            d.name().len()
        });
        assert_eq!(name_len, Some(5));

        let ticks = reg.find("timer0", |d| d.read_ticks());
        assert_eq!(ticks, Some(42));

        // Unknown name -> None.
        assert!(reg.find("does-not-exist", |_| ()).is_none());
    }

    #[test]
    fn registry_rejects_overflow() {
        let mut reg = DriverRegistry::new();
        let mut accepted = 0;
        for _ in 0..MAX_DRIVERS {
            // SAFETY (test): re-borrowing a static, no live aliasing borrow.
            if reg.register(unsafe { &mut *core::ptr::addr_of_mut!(FAKE_UART) }).is_ok() {
                accepted += 1;
            }
        }
        assert_eq!(accepted, MAX_DRIVERS);
        // SAFETY (test): re-borrowing a static, no live aliasing borrow.
        assert_eq!(
            reg.register(unsafe { &mut *core::ptr::addr_of_mut!(FAKE_UART) }),
            Err(DriverError::Busy)
        );
    }

    #[test]
    fn init_failure_is_counted_not_fatal() {
        struct BrokenDriver;
        impl Driver for BrokenDriver {
            fn name(&self) -> &'static str {
                "broken"
            }
            fn class(&self) -> DeviceClass {
                DeviceClass::Timer
            }
            fn init(&mut self) -> Result<(), DriverError> {
                Err(DriverError::InitFailed)
            }
        }
        static mut BROKEN: BrokenDriver = BrokenDriver;

        let mut reg = DriverRegistry::new();
        // SAFETY (test): re-borrowing a static mut, no live aliasing borrow.
        let _ = reg.register(unsafe { &mut *core::ptr::addr_of_mut!(BROKEN) });
        assert_eq!(reg.init_all(), 0); // failed init is counted, not fatal
        assert_eq!(reg.len(), 1); // but the driver stays registered
    }
}
