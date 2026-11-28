.text
.p2align 4
fiber_symbol jaic_fiber_switch
    aghi %r15, -144
    stmg %r6, %r14, 0(%r15)
    std %f8, 72(%r15)
    std %f9, 80(%r15)
    std %f10, 88(%r15)
    std %f11, 96(%r15)
    std %f12, 104(%r15)
    std %f13, 112(%r15)
    std %f14, 120(%r15)
    std %f15, 128(%r15)
    stg %r15, 0(%r2)
    lg %r15, 0(%r3)
    lmg %r6, %r14, 0(%r15)
    ld %f8, 72(%r15)
    ld %f9, 80(%r15)
    ld %f10, 88(%r15)
    ld %f11, 96(%r15)
    ld %f12, 104(%r15)
    ld %f13, 112(%r15)
    ld %f14, 120(%r15)
    ld %f15, 128(%r15)
    aghi %r15, 144
    br %r14
.p2align 4
fiber_symbol jaic_fiber_prepare
    lghi %r0, -8
    ngr %r2, %r0
    aghi %r2, -144
    xc 0(144, %r2), 0(%r2)
    stg %r3, 0(%r2)
    stg %r4, 8(%r2)
    larl %r1, .Ljaic_fiber_trampoline
    stg %r1, 64(%r2)
    br %r14
.p2align 4
fiber_trampoline_label
    aghi %r15, -160
    lgr %r2, %r7
    basr %r14, %r6
    .short 0
