MEMORY {
    RAM : ORIGIN = 0x80000000, LENGTH = 128M
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);

ENTRY(_start)

SECTIONS {
    . = 0x80000000;
    
    .text.init : {
        KEEP(*(.text._start));
    } > RAM
    
    .text : {
        *(.text .text.*);
    } > RAM

    .rodata : {
        *(.rodata .rodata.*);
    } > RAM

    .data : {
        *(.data .data.*);
    } > RAM

    .bss : {
        *(.bss .bss.*);
        *(COMMON);
    } > RAM
}
