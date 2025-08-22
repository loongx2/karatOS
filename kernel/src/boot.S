.syntax unified
.cpu cortex-m3
.thumb

.section .vector_table, "a", %progbits
.type vector_table, %object
.size vector_table, .-vector_table

vector_table:
    .word  _stack_start     // Initial stack pointer
    .word  Reset_Handler+1  // Reset handler (Thumb mode)
    .word  NMI_Handler+1    // NMI handler
    .word  HardFault_Handler+1  // Hard fault handler
    .word  0                // Reserved
    .word  0                // Reserved
    .word  0                // Reserved
    .word  0                // Reserved
    .word  0                // Reserved
    .word  0                // Reserved
    .word  0                // Reserved
    .word  SVC_Handler+1    // SVCall handler
    .word  0                // Reserved
    .word  0                // Reserved
    .word  PendSV_Handler+1 // PendSV handler
    .word  SysTick_Handler+1  // SysTick handler

.text
.thumb_func
.global Reset_Handler
Reset_Handler:
    // Set up stack pointer
    ldr r0, =_stack_start
    mov sp, r0
    
    // Call main
    bl main
    
    // If main returns, loop forever
1:  b 1b

.thumb_func
NMI_Handler:
    b NMI_Handler

.thumb_func
HardFault_Handler:
    b HardFault_Handler

.thumb_func
SVC_Handler:
    b SVC_Handler

.thumb_func
PendSV_Handler:
    b PendSV_Handler

.thumb_func
SysTick_Handler:
    b SysTick_Handler
