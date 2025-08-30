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

    .data : {
        *(.data .data.*);
    } > RAM

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

    /DISCARD/ : {
        *(.eh_frame);
    }
}
