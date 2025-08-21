/* RISC-V memory.x for QEMU virt machine - Simple layout */
MEMORY {
  /* QEMU virt machine: Simple layout in low RAM */
  REGION_TEXT : ORIGIN = 0x80000000, LENGTH = 4M
  REGION_RODATA : ORIGIN = 0x80400000, LENGTH = 1M  
  REGION_DATA : ORIGIN = 0x80500000, LENGTH = 1M
  REGION_BSS : ORIGIN = 0x80600000, LENGTH = 1M
  REGION_HEAP : ORIGIN = 0x80700000, LENGTH = 1M
  REGION_STACK : ORIGIN = 0x80800000, LENGTH = 1M
}

_max_hart_id = 0;
_hart_stack_size = 4K;
_heap_size = 1024;
