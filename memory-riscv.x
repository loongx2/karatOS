/* RISC-V Memory Layout for QEMU virt machine */
MEMORY
{
  /* QEMU virt machine layout - using riscv-rt region names */
  REGION_TEXT : ORIGIN = 0x20000000, LENGTH = 2M
  REGION_RODATA : ORIGIN = 0x20000000, LENGTH = 2M  
  REGION_DATA : ORIGIN = 0x80000000, LENGTH = 128M
  REGION_BSS : ORIGIN = 0x80000000, LENGTH = 128M
  REGION_HEAP : ORIGIN = 0x80000000, LENGTH = 128M
  REGION_STACK : ORIGIN = 0x80000000, LENGTH = 128M
}

/* Required by riscv-rt */
_max_hart_id = 0;
_hart_stack_size = 2K;
_heap_size = 0;
