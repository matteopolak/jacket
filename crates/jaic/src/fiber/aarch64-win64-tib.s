// Windows keeps the stack bounds in the thread environment block, through x18: StackBase +8,
// StackLimit +16 and DeallocationStack +0x1478 (an offset too big for ldr's immediate, so it
// goes through x11).
.macro fiber_save_tib
    ldr x10, [x18, #8]
    str x10, [x0, #16]
    ldr x10, [x18, #16]
    str x10, [x0, #8]
    mov x11, #0x1478
    ldr x10, [x18, x11]
    str x10, [x0, #24]
.endm
.macro fiber_load_tib
    ldr x10, [x1, #16]
    str x10, [x18, #8]
    ldr x10, [x1, #8]
    str x10, [x18, #16]
    mov x11, #0x1478
    ldr x10, [x1, #24]
    str x10, [x18, x11]
.endm
