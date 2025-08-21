/* RISC-V memory.x for QEMU virt machine - Fixed layout */
MEMORY {
  /* QEMU virt machine: All in RAM starting at 0x80000000 */
  REGION_TEXT   : ORIGIN = 0x80000000, LENGTH = 4M
  REGION_RODATA : ORIGIN = 0x80400000, LENGTH = 4M  
  REGION_DATA   : ORIGIN = 0x80800000, LENGTH = 4M
  REGION_BSS    : ORIGIN = 0x80C00000, LENGTH = 4M
  REGION_HEAP   : ORIGIN = 0x81000000, LENGTH = 16M
  REGION_STACK  : ORIGIN = 0x82000000, LENGTH = 16M
}

_max_hart_id = 0;
_hart_stack_size = 4K;
_heap_size = 1024;
