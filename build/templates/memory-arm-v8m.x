/* karatOS memory layout - ARMv8-M baseline (Cortex-M33/M55) */
/* Default region: MPS3-AN547 secure aliases (QEMU -M mps3-an547)     */
/*   FLASH (secure alias of RAM block 0): 0x10000000, 2 MB            */
/*   SRAM  (secure alias of RAM block 0): 0x30000000, 2 MB            */
/* For Musca-B1 override with flash 0x00000000 / sram 0x20000000.     */

MEMORY
{
    FLASH (rx)  : ORIGIN = 0x10000000, LENGTH = 2M
    RAM   (rwx) : ORIGIN = 0x30000000, LENGTH = 2M
}

__STACK_TOP = ORIGIN(RAM) + LENGTH(RAM);

ENTRY(_reset)

SECTIONS
{
    .text : ALIGN(4)
    {
        KEEP(*(.isr_vector));          /* Vector table at flash base  */
        *(.text .text.*);
        *(.rodata .rodata.*);
        . = ALIGN(4);
        _etext = .;
    } > FLASH

    .data : ALIGN(4)
    {
        _sdata = .;
        *(.data .data.*);
        . = ALIGN(4);
        _edata = .;
    } > RAM AT > FLASH
    _sidata = LOADADDR(.data);

    .bss (NOLOAD) : ALIGN(4)
    {
        _sbss = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(4);
        _ebss = .;
    } > RAM

    .heap (NOLOAD) : ALIGN(8)
    {
        _sheap = .;
        . = . + 0x2000;                /* 8 KB heap budget            */
        _eheap = .;
    } > RAM

    ._user_heap_stack (NOLOAD) : ALIGN(8)
    {
        . = . + 0x1000;                /* 4 KB main stack             */
        . = ALIGN(8);
    } > RAM

    /DISCARD/ : { *(.eh_frame) *(.ARM.exidx*) }
}

/* Keep total static footprint well under the 64 KB ROM/SRAM budget */
ASSERT((_etext - ORIGIN(FLASH)) < 64K, "ERROR: .text+.rodata exceeds 64KB")
ASSERT((_ebss - _sbss) < 64K, "ERROR: .bss exceeds 64KB")
