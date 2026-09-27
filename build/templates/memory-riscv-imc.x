/* karatOS memory layout - RISC-V RV32IMC (no atomics, e.g. ESP32-C3 class) */
/* Compatible with riscv-rt crate requirements                              */

MEMORY {
    RAM : ORIGIN = 0x80000000, LENGTH = 128M
}

/* Stack at the end of RAM */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
PROVIDE(_stack_start = _stack_start);

/* Entry point for riscv-rt */
ENTRY(_start)

SECTIONS {
    .text : ALIGN(4) {
        KEEP(*(.init));
        KEEP(*(.init.rust));
        *(.text .text.*);
        *(.rodata .rodata.*);
        . = ALIGN(4);
        _etext = .;
    } > RAM

    /* riscv-rt's trap prologue: kept + aliased here because our custom
       script replaces riscv-rt's link.x.in. */
    .trap : ALIGN(4) {
        KEEP(*(.trap .trap.*));
        _etrap = .;
    } > RAM

    .data : ALIGN(4) {
        _sdata = .;
        *(.data .data.*);
        . = ALIGN(4);
        _edata = .;
    } > RAM

    /* RAM-only image: load address == run address (no-op copy in riscv-rt). */
    _sidata = _sdata;

    /* riscv-rt names its trap prologue `default_start_trap`; alias it for
       our mtvec setup (riscv-rt's link.x.in is not used in this project). */
    PROVIDE(_start_trap = default_start_trap);

    /* ------------------------------------------------------------------------
     * riscv-rt weak-symbol defaults (normally in link.x.in, which this
     * project replaces with its own templates). Strong symbols from the
     * kernel crate (MachineTimer, _setup_interrupts, _mp_hook, __pre_init)
     * automatically take precedence over these PROVIDE defaults.
     * ---------------------------------------------------------------------- */
    PROVIDE(InstructionMisaligned = ExceptionHandler);
    PROVIDE(InstructionFault      = ExceptionHandler);
    PROVIDE(IllegalInstruction    = ExceptionHandler);
    PROVIDE(Breakpoint            = ExceptionHandler);
    PROVIDE(LoadMisaligned        = ExceptionHandler);
    PROVIDE(LoadFault             = ExceptionHandler);
    PROVIDE(StoreMisaligned       = ExceptionHandler);
    PROVIDE(StoreFault            = ExceptionHandler);
    PROVIDE(UserEnvCall           = ExceptionHandler);
    PROVIDE(SupervisorEnvCall     = ExceptionHandler);
    PROVIDE(MachineEnvCall        = ExceptionHandler);
    PROVIDE(InstructionPageFault  = ExceptionHandler);
    PROVIDE(LoadPageFault         = ExceptionHandler);
    PROVIDE(StorePageFault        = ExceptionHandler);
    PROVIDE(SupervisorSoft        = DefaultHandler);
    PROVIDE(MachineSoft           = DefaultHandler);
    PROVIDE(SupervisorTimer       = DefaultHandler);
    PROVIDE(MachineTimer          = DefaultHandler);
    PROVIDE(SupervisorExternal    = DefaultHandler);
    PROVIDE(MachineExternal       = DefaultHandler);
    PROVIDE(DefaultHandler        = DefaultInterruptHandler);
    PROVIDE(ExceptionHandler      = DefaultExceptionHandler);

    .bss (NOLOAD) : ALIGN(4) {
        _sbss = .;
        *(.bss .bss.*);
        *(COMMON);
        . = ALIGN(4);
        _ebss = .;
    } > RAM

    .heap (NOLOAD) : ALIGN(4) {
        _sheap = .;
        . = . + 0x2000; /* 8K heap */
        _eheap = .;
    } > RAM

        /* KAPI export table at a FIXED address so loadable modules bind to it
       at their own link time (karatos-kapi::KAPI_ADDR_RISCV). */
    .kapi 0x80010000 : {
        KEEP(*(.kapi .kapi.*));
    } > RAM

    /DISCARD/ : {
        *(.eh_frame);
    }
}

/* Static image must fit the 64 KB ROM/SRAM budget */
ASSERT((_etext - ORIGIN(RAM)) < 64K, "ERROR: .text+.rodata exceeds 64KB")
ASSERT((_ebss - _sbss) < 64K, "ERROR: .bss exceeds 64KB")
