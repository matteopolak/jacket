.text
.p2align 4
fiber_symbol jaic_fiber_switch
    pushq %rbp
    pushq %rbx
    pushq %rdi
    pushq %rsi
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15
    subq $168, %rsp
    movups %xmm6, 0(%rsp)
    movups %xmm7, 16(%rsp)
    movups %xmm8, 32(%rsp)
    movups %xmm9, 48(%rsp)
    movups %xmm10, 64(%rsp)
    movups %xmm11, 80(%rsp)
    movups %xmm12, 96(%rsp)
    movups %xmm13, 112(%rsp)
    movups %xmm14, 128(%rsp)
    movups %xmm15, 144(%rsp)
    movq %gs:8, %rax
    movq %rax, 16(%rcx)
    movq %gs:16, %rax
    movq %rax, 8(%rcx)
    movq %gs:0x1478, %rax
    movq %rax, 24(%rcx)
    movq %rsp, (%rcx)
    movq (%rdx), %rsp
    movq 16(%rdx), %rax
    movq %rax, %gs:8
    movq 8(%rdx), %rax
    movq %rax, %gs:16
    movq 24(%rdx), %rax
    movq %rax, %gs:0x1478
    movups 0(%rsp), %xmm6
    movups 16(%rsp), %xmm7
    movups 32(%rsp), %xmm8
    movups 48(%rsp), %xmm9
    movups 64(%rsp), %xmm10
    movups 80(%rsp), %xmm11
    movups 96(%rsp), %xmm12
    movups 112(%rsp), %xmm13
    movups 128(%rsp), %xmm14
    movups 144(%rsp), %xmm15
    addq $168, %rsp
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rsi
    popq %rdi
    popq %rbx
    popq %rbp
    ret
.p2align 4
fiber_symbol jaic_fiber_prepare
    andq $-16, %rcx
    fiber_lea_trampoline %rax
    movq %rax, -8(%rcx)
    movq $0, -16(%rcx)
    movq $0, -24(%rcx)
    movq $0, -32(%rcx)
    movq $0, -40(%rcx)
    movq %rdx, -48(%rcx)
    movq %r8, -56(%rcx)
    movq $0, -64(%rcx)
    movq $0, -72(%rcx)
    leaq -240(%rcx), %rax
    ret
.p2align 4
fiber_trampoline_label
    movq %r13, %rcx
    subq $32, %rsp
    callq *%r12
    ud2
