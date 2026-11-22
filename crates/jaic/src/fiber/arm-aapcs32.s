.text
.arm
.p2align 2
fiber_symbol jaic_fiber_switch
    push {r4, r5, r6, r7, r8, r9, r10, r11, lr}
    vpush {d8, d9, d10, d11, d12, d13, d14, d15}
    str sp, [r0]
    ldr sp, [r1]
    vpop {d8, d9, d10, d11, d12, d13, d14, d15}
    pop {r4, r5, r6, r7, r8, r9, r10, r11, lr}
    bx lr
.p2align 2
fiber_symbol jaic_fiber_prepare
    bic r0, r0, #7
    sub r0, r0, #100
    mov r3, #0
    @ d8-d15 (offsets 0-63) are left as they were; r4 = entry, r5 = argument, r6-r11 = 0.
    str r1, [r0, #64]
    str r2, [r0, #68]
    str r3, [r0, #72]
    str r3, [r0, #76]
    str r3, [r0, #80]
    str r3, [r0, #84]
    str r3, [r0, #88]
    str r3, [r0, #92]
    adr r3, .Ljaic_fiber_trampoline
    str r3, [r0, #96]
    bx lr
.p2align 2
fiber_trampoline_label
    mov r0, r5
    blx r4
    udf #0
