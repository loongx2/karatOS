/* RISC-V memory layout for QEMU virt machine */
/* Compatible with riscv-rt crate requirements */

MEMORY {
    RAM : ORIGIN = 0x80000000, LENGTH = 128M
}

/* Stack at the end of RAM */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
PROVIDE(_stack_start = _stack_start);

/* Entry point for riscv-rt */
ENTRY(_start)

SECTIONS {
    .text : {
        KEEP(*(.init));
        KEEP(*(.init.rust));
        *(.text .text.*);
    } > RAM

    .rodata : {
        *(.rodata .rodata.*);
    } > RAM

    /* riscv-rt's trap prologue (default_start_trap). Kept explicitly and
       aliased to _start_trap because our custom script replaces riscv-rt's
       link.x.in, which normally does both jobs. */
    .trap : ALIGN(4) {
        KEEP(*(.trap .trap.*));
    } > RAM

    .data : {
        _sdata = .;
        *(.data .data.*);
        . = ALIGN(4);
        _edata = .;
    } > RAM

    /* RAM-only image: the load address of .data equals its run address, so
       riscv-rt's .data copy loop degenerates to a no-op self-copy. */
    _sidata = _sdata;

    /* riscv-rt names its trap prologue `default_start_trap` and normally
       aliases it via link.x.in; we own the linker script, so the alias is
       ours to make (referenced by _setup_interrupts / mtvec setup). */
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

    .bss (NOLOAD) : {
        . = ALIGN(4);
        _sbss = .;
        *(.bss .bss.*);
        *(COMMON);
        . = ALIGN(4);
        _ebss = .;
    } > RAM

    /* Heap area (optional) */
    .heap (NOLOAD) : {
        . = ALIGN(4);
        _sheap = .;
        . = . + 0x1000; /* 4K heap */
        . = ALIGN(4);
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
