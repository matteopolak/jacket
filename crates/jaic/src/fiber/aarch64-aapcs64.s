.text
.p2align 2
fiber_symbol jaic_fiber_switch
    sub sp, sp, #160
    stp x19, x20, [sp, #0]
    stp x21, x22, [sp, #16]
    stp x23, x24, [sp, #32]
    stp x25, x26, [sp, #48]
    stp x27, x28, [sp, #64]
    stp x29, x30, [sp, #80]
    stp d8, d9, [sp, #96]
    stp d10, d11, [sp, #112]
    stp d12, d13, [sp, #128]
    stp d14, d15, [sp, #144]
    fiber_save_tib
    mov x9, sp
    str x9, [x0]
    ldr x9, [x1]
    mov sp, x9
    fiber_load_tib
    ldp x19, x20, [sp, #0]
    ldp x21, x22, [sp, #16]
    ldp x23, x24, [sp, #32]
    ldp x25, x26, [sp, #48]
    ldp x27, x28, [sp, #64]
    ldp x29, x30, [sp, #80]
    ldp d8, d9, [sp, #96]
    ldp d10, d11, [sp, #112]
    ldp d12, d13, [sp, #128]
    ldp d14, d15, [sp, #144]
    add sp, sp, #160
    ret
.p2align 2
fiber_symbol jaic_fiber_prepare
    and x0, x0, #0xfffffffffffffff0
    sub x0, x0, #160
    stp xzr, xzr, [x0, #16]
    stp xzr, xzr, [x0, #32]
    stp xzr, xzr, [x0, #48]
    stp xzr, xzr, [x0, #64]
    stp xzr, xzr, [x0, #96]
    stp xzr, xzr, [x0, #112]
    stp xzr, xzr, [x0, #128]
    stp xzr, xzr, [x0, #144]
    stp x1, x2, [x0, #0]
    fiber_adr_trampoline x9
    stp xzr, x9, [x0, #80]
    ret
.p2align 2
fiber_trampoline_label
    mov x0, x20
    blr x19
    brk #0
