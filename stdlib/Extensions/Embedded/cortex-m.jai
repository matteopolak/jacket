// What a Cortex-M core needs to start a program: the vector table, a reset handler that turns
// the floating-point unit on and calls the program's `jai_start`, a fault handler that ends the
// run with status 1, and the semihosting trap (`semihost`, see semihosting.jai).
//
// The core reads the table from address 0 (or `VTOR`): the initial stack pointer, then the reset
// handler. The linker script puts `.vector_table` first (see `Memory_Map`), so the image must be
// linked at the address the core boots from.
//
//     0x00  initial stack pointer     0x14  bus fault        0x2c  SVCall
//     0x04  reset                     0x18  usage fault      0x38  PendSV
//     0x08  NMI                       0x1c  (reserved)       0x3c  SysTick
//     0x0c  hard fault                0x10  memory fault     0x40  device interrupts...
//
// A program that uses interrupts places its own table with `@Section(".vector_table")` instead
// and calls `start_semihosted` (or its own startup) from its reset handler.

cortex_m_code :: #string ASM
.syntax unified
.thumb

.section .vector_table, "a"
.word __stack_top
.word _start
.word cortex_m_fault   // NMI
.word cortex_m_fault   // hard fault
.word cortex_m_fault   // memory management fault
.word cortex_m_fault   // bus fault
.word cortex_m_fault   // usage fault
.word 0, 0, 0, 0
.word cortex_m_fault   // SVCall
.word 0, 0
.word cortex_m_fault   // PendSV
.word cortex_m_fault   // SysTick

.section .text.boot, "ax"
.global _start
.thumb_func
_start:
    ldr r0, =0xE000ED88   // CPACR: full access to the FPU (CP10 and CP11)
    ldr r1, [r0]
    orr r1, r1, #(0xF << 20)
    str r1, [r0]
    dsb
    isb
    bl  jai_start
1:  b   1b

// A fault ends the run with status 1: SYS_EXIT_EXTENDED with ADP_Stopped_ApplicationExit.
.thumb_func
cortex_m_fault:
    ldr r0, =0x20026
    movs r1, #1
    push {r0, r1}
    mov r1, sp
    movs r0, #0x20
    bkpt 0xab
2:  b   2b

.text
.global semihost
.thumb_func
semihost:
    bkpt 0xab
    bx  lr
ASM; @Module_Asm
