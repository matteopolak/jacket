.text
.p2align 4
fiber_symbol jaic_fiber_switch
    pushq %rbp
    pushq %rbx
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15
    movq %rsp, (%rdi)
    movq (%rsi), %rsp
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbx
    popq %rbp
    ret
.p2align 4
fiber_symbol jaic_fiber_prepare
    andq $-16, %rdi
    fiber_lea_trampoline %rax
    movq %rax, -8(%rdi)
    movq $0, -16(%rdi)
    movq $0, -24(%rdi)
    movq %rsi, -32(%rdi)
    movq %rdx, -40(%rdi)
    movq $0, -48(%rdi)
    movq $0, -56(%rdi)
    leaq -56(%rdi), %rax
    ret
.p2align 4
fiber_trampoline_label
    movq %r13, %rdi
    callq *%r12
    ud2
