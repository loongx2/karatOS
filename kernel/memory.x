MEMORY {
    RAM : ORIGIN = 0x80000000, LENGTH = 128M
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);

ENTRY(_start)

SECTIONS {
    . = ORIGIN(RAM);
    
    .text.init : {
        KEEP(*(.text._start));
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
    
    /* Discard unwanted sections */
    /DISCARD/ : {
        *(.eh_frame);
    }
}
