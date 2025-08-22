MEMORY {
    FLASH : ORIGIN = 0x00000000, LENGTH = 256K
    RAM : ORIGIN = 0x20000000, LENGTH = 64K
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);

ENTRY(_start)

SECTIONS {
    . = ORIGIN(FLASH);
    
    .text : {
        KEEP(*(.text._start));
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
