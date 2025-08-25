/* Simple RISC-V memory layout for QEMU virt machine */
MEMORY {
    RAM : ORIGIN = 0x80000000, LENGTH = 128M
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);

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
        *(.bss .bss.*);
        *(COMMON);
    } > RAM

    /DISCARD/ : {
        *(.eh_frame);
    }
}
