MEMORY {
    FLASH : ORIGIN = 0x00000000, LENGTH = 128M
    RAM : ORIGIN = 0x40000000, LENGTH = 128M
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);

SECTIONS {
    .vector_table ORIGIN(FLASH) : {
        KEEP(*(.vector_table));
    } > FLASH

    .text : {
        *(.text .text.*);
    } > FLASH

    .rodata : {
        *(.rodata .rodata.*);
    } > FLASH

    .data : {
        *(.data .data.*);
    } > RAM AT > FLASH

    .bss : {
        *(.bss .bss.*);
        *(COMMON);
    } > RAM

    /* Discard unwanted sections */
    /DISCARD/ : {
        *(.eh_frame);
    }
}
