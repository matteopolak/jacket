// EL1 reset entry for QEMU virt: set the stack, call Jai, exit through semihosting.
.section .text.boot, "ax"
.global _start
_start:
    ldr x0, =__stack_top
    mov sp, x0
    bl  jai_start
1:  b   1b

.global semihost
semihost:
    hlt #0xf000
    ret
