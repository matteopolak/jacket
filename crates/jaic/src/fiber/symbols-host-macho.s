.macro fiber_symbol name
.globl _\name
_\name:
.endm
.macro fiber_trampoline_label
Ljaic_fiber_trampoline:
.endm
.macro fiber_lea_trampoline reg
    leaq Ljaic_fiber_trampoline(%rip), \reg
.endm
.macro fiber_adr_trampoline reg
    adr \reg, Ljaic_fiber_trampoline
.endm
