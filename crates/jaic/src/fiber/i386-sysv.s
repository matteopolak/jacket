.text
.p2align 4
fiber_symbol jaic_fiber_switch
    movl 4(%esp), %eax
    movl 8(%esp), %edx
    pushl %ebp
    pushl %ebx
    pushl %esi
    pushl %edi
    movl %esp, (%eax)
    movl (%edx), %esp
    popl %edi
    popl %esi
    popl %ebx
    popl %ebp
    ret
.p2align 4
fiber_symbol jaic_fiber_prepare
    movl 4(%esp), %eax
    andl $-16, %eax
    call 1f
1:
    popl %ecx
    addl $(.Ljaic_fiber_trampoline - 1b), %ecx
    movl %ecx, -16(%eax)
    movl $0, -20(%eax)
    movl 12(%esp), %ecx
    movl %ecx, -24(%eax)
    movl 8(%esp), %ecx
    movl %ecx, -28(%eax)
    movl $0, -32(%eax)
    subl $32, %eax
    ret
.p2align 4
fiber_trampoline_label
    pushl %ebx
    calll *%esi
    ud2
