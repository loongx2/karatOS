/* ARM Cortex-M3 memory layout for LM3S6965EVB in QEMU */
/* Compatible with cortex-m-rt crate requirements */

MEMORY {
    /* LM3S6965EVB has 256KB Flash and 64KB RAM */
    FLASH : ORIGIN = 0x00000000, LENGTH = 256K
    RAM   : ORIGIN = 0x20000000, LENGTH = 64K
}

/* Stack at the end of RAM */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
