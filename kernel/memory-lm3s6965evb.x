/* Linker script for LM3S6965EVB board (Cortex-M3) */
/* Compatible with cortex-m-rt crate requirements */

MEMORY
{
  /* LM3S6965EVB: 256K Flash at 0x00000000, 64K SRAM at 0x20000000 */
  FLASH : ORIGIN = 0x00000000, LENGTH = 256K
  RAM : ORIGIN = 0x20000000, LENGTH = 64K
}

/* Stack grows downward from end of RAM */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

/* Entry point - cortex-m-rt will handle this */
ENTRY(Reset);

SECTIONS
{
  /* Vector table must be at start of FLASH and properly aligned */
  /* ARM Cortex-M3 requires minimum 32-word (128-byte) alignment */
  .vector_table ORIGIN(FLASH) :
  {
    /* First entry: initial stack pointer */
    LONG(_stack_start);
    /* Reset handler and exception vectors */
    KEEP(*(.vector_table.reset_vector));
    KEEP(*(.vector_table.exceptions)); 
    KEEP(*(.vector_table.interrupts));
  } > FLASH

  /* Code and constants section */
  .text :
  {
    *(.Reset);
    *(.text .text.*);
  } > FLASH

  .rodata :
  {
    *(.rodata .rodata.*);
  } > FLASH

  /* Initialized data section */
  .data : AT(ADDR(.rodata) + SIZEOF(.rodata))
  {
    . = ALIGN(4);
    __sdata = .;
    *(.data .data.*);
    . = ALIGN(4);
    __edata = .;
  } > RAM

  /* Uninitialized data section */
  .bss :
  {
    . = ALIGN(4);
    __sbss = .;
    *(.bss .bss.*);
    *(COMMON);
    . = ALIGN(4);
    __ebss = .;
  } > RAM

  /* Data load address for initialization */
  __sidata = LOADADDR(.data);

  /* Heap area (optional) */
  .heap (NOLOAD) :
  {
    . = ALIGN(4);
    __sheap = .;
    . = . + 0x400; /* 1K heap */
    . = ALIGN(4);
    __eheap = .;
  } > RAM

  /* Stack check (ensure we don't overflow) */
  .stack (NOLOAD) :
  {
    . = . + 0x400; /* 1K stack reserved */
  } > RAM

  /* Remove information from the standard libraries */
  /DISCARD/ :
  {
    libc.a ( * )
    libm.a ( * )
    libgcc.a ( * )
    *(.ARM.exidx* .gnu.linkonce.armexidx.*)
  }
}
