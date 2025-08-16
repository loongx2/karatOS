// Simple ARM assembly entry point
.syntax unified
.cpu cortex-m3
.thumb

.section .vector_table, "a"
.word _stack_start          // Stack pointer (0x0)
.word reset_handler + 1     // Reset handler (0x4) - +1 for Thumb mode

.section .text
.thumb_func
.global reset_handler
reset_handler:
    // Jump to main function
    bl main
    // If main returns, loop forever
1:  b 1b

.thumb_func
.global _start
_start:
    b reset_handler
