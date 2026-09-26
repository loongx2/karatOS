/* karatOS memory layout - RISC-V RV64GC (QEMU virt, qemu-system-riscv64) */
/* Compatible with riscv-rt crate requirements                            */

MEMORY {
    RAM : ORIGIN = 0x80000000, LENGTH = 128M
}

/* Stack at the end of RAM */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
PROVIDE(_stack_start = _stack_start);

/* Entry point for riscv-rt */
ENTRY(_start)

SECTIONS {
    .text : ALIGN(8) {
        KEEP(*(.init));
        KEEP(*(.init.rust));
        *(.text .text.*);
        *(.rodata .rodata.*);
        . = ALIGN(8);
        _etext = .;
    } > RAM

    .data : ALIGN(8) {
        _sdata = .;
        *(.data .data.*);
        . = ALIGN(8);
        _edata = .;
    } > RAM

    .bss (NOLOAD) : ALIGN(8) {
        _sbss = .;
        *(.bss .bss.*);
        *(COMMON);
        . = ALIGN(8);
        _ebss = .;
    } > RAM

    /* Heap area (budgeted small to respect 64 KB footprint target) */
    .heap (NOLOAD) : ALIGN(8) {
        _sheap = .;
        . = . + 0x2000; /* 8K heap */
        _eheap = .;
    } > RAM

    /DISCARD/ : {
        *(.eh_frame);
    }
}

/* Static image must fit the 64 KB ROM/SRAM budget */
ASSERT((_etext - ORIGIN(RAM)) < 64K, "ERROR: .text+.rodata exceeds 64KB")
ASSERT((_ebss - _sbss) < 64K, "ERROR: .bss exceeds 64KB")
