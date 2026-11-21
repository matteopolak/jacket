// EL1 reset entry for QEMU virt: set the stack, call Jai, exit through semihosting.
.section .text.boot, "ax"
.global _start
_start:
    // Floating point and SIMD trap at reset; enable them (CPACR_EL1.FPEN).
    mrs x0, cpacr_el1
    orr x0, x0, #(3 << 20)
    msr cpacr_el1, x0
    isb
    ldr x0, =__stack_top
    mov sp, x0
    bl  jai_start
1:  b   1b

.global semihost
semihost:
    hlt #0xf000
    ret
