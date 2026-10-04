// Vector #asm instructions are lowered lane by lane: moves, float and integer arithmetic,
// compares, shifts, shuffles, conversions, broadcasts, masks and gathers.
#import "Basic";

fail :: (code: int, what: string) { print("FAIL %: %\n", code, what); exit(xx code); }

main :: () {
    // Float arithmetic, 128-bit legacy form (dst op= src).
    a := float32.[1, 2, 3, 4];
    b := float32.[10, 20, 30, 40];
    pa, pb := a.data, b.data;
    #asm {
        movups.x v:, [pa];
        movups.x w:, [pb];
        mulps.x v, w;
        subps.x v, w;
        movups.x [pa], v;
    }
    if a[0] != 0 || a[1] != 20 || a[3] != 120 fail(1, "mulps/subps");

    // Three-operand AVX form; the default size is 256 bits.
    c := float32.[1, 4, 9, 16, 25, 36, 49, 64];
    d: [8] float32;
    pc, pd := c.data, d.data;
    #asm AVX {
        movups x:, [pc];
        sqrtps y:, x;
        maxps z:, x, y;
        divps z, z, y;
        movups [pd], z;
    }
    if d[0] != 1 || d[1] != 2 || d[7] != 8 fail(2, "sqrtps/maxps/divps");

    // Scalar lane ops keep the other lanes.
    s := float.[2, 7, 7, 7];
    t := float.[3, 0, 0, 0];
    ps, pt := s.data, t.data;
    #asm {
        movups.x v:, [ps];
        movss w:, [pt];
        addss v, w;
        movups.x [ps], v;
    }
    if s[0] != 5 || s[1] != 7 fail(3, "addss");

    // Integer lanes: wrapping adds, compares, min/max, abs, logic.
    i := s32.[1, -2, 3, -4];
    j := s32.[0x7fff_ffff, 5, 3, -10];
    out: [4] s32;
    pi, pj, po := i.data, j.data, out.data;
    #asm {
        movdqu.x v:, [pi];
        movdqu.x w:, [pj];
        paddd.x v, w;
        movdqu.x [po], v;
    }
    if out[0] != -0x8000_0000 || out[1] != 3 || out[3] != -14 fail(4, "paddd");
    #asm AVX {
        movdqu.x v:, [pi];
        movdqu.x w:, [pj];
        pcmpgtd.x e:, v, w;
        movdqu.x [po], e;
    }
    if out[0] != 0 || out[1] != 0 || out[3] != -1 fail(5, "pcmpgtd");
    #asm AVX {
        movdqu.x v:, [pi];
        movdqu.x w:, [pj];
        pminsd.x m:, v, w;
        pabsd.x m, m;
        movdqu.x [po], m;
    }
    if out[0] != 1 || out[1] != 2 || out[3] != 10 fail(6, "pminsd/pabsd");
    #asm AVX {
        pxor.x z:, z, z;
        pcmpeqd.x ones:, z, z;
        pandn.x r:, z, ones;
        movdqu.x [po], r;
    }
    if out[0] != -1 || out[3] != -1 fail(7, "pxor/pcmpeqd/pandn");

    // Shifts by immediate.
    h := u16.[1, 0x8000, 3, 4, 5, 6, 7, 0xffff];
    ph := h.data;
    #asm {
        movdqu.x v:, [ph];
        psllw.x v, 4;
        movdqu.x [ph], v;
    }
    if h[0] != 16 || h[1] != 0 || h[7] != 0xfff0 fail(8, "psllw");
    #asm AVX {
        movdqu.x v:, [ph];
        psraw.x v, v, 20;
        movdqu.x [ph], v;
    }
    if h[0] != 0 || h[7] != 0xffff fail(9, "psraw oversized");

    // Shuffles.
    q := s32.[10, 20, 30, 40];
    pq := q.data;
    #asm {
        movdqu.x v:, [pq];
        pshufd.x w:, v, 0b00_01_10_11;
        movdqu.x [po], w;
    }
    if out[0] != 40 || out[1] != 30 || out[3] != 10 fail(10, "pshufd");

    // Conversions: round half to even, truncation, out of range.
    f := float32.[2.5, -3.5, 1.7, 3e10];
    pf := f.data;
    #asm {
        movups.x v:, [pf];
        cvtps2dq.x w:, v;
        movdqu.x [po], w;
    }
    if out[0] != 2 || out[1] != -4 || out[2] != 2 || out[3] != -0x8000_0000 fail(11, "cvtps2dq");
    #asm {
        movups.x v:, [pf];
        cvttps2dq.x w:, v;
        cvtdq2ps.x x:, w;
        movups.x [pf], x;
    }
    if f[0] != 2 || f[1] != -3 || f[2] != 1 fail(12, "cvttps2dq/cvtdq2ps");

    // movd/movq with general-purpose registers, movmskps, broadcast.
    value: u64 = 0x1234_5678_9abc_def0;
    low: u64;
    mask: u64;
    #asm {
        movq v:, value;
        movd low, v;
    }
    if low != 0x9abc_def0 fail(13, "movq/movd");
    signs := float32.[-1, 2, -3, 4];
    pm := signs.data;
    #asm {
        movups.x v:, [pm];
        movmskps mask, v;
    }
    if mask != 0b0101 fail(14, "movmskps");
    one: float32 = 1.5;
    pone := *one;
    fill: [8] float32;
    pfill := fill.data;
    #asm AVX {
        broadcastss v:, [pone];
        movups [pfill], v;
    }
    if fill[0] != 1.5 || fill[7] != 1.5 fail(15, "broadcastss");

    // A gather that only loads the lanes its mask selects.
    src := s32.[100, 101, 102, 103, 104, 105, 106, 107];
    idx := s32.[7, 0, 3, 3];
    sel := s32.[-1, 0, -1, -1];
    got := s32.[9, 9, 9, 9];
    psrc, pidx, psel, pgot := src.data, idx.data, sel.data, got.data;
    #asm AVX2 {
        movdqu.x vi:, [pidx];
        movdqu.x vm:, [psel];
        movdqu.x vd:, [pgot];
        pgatherdd.x vd, [psrc + vi*4], vm;
        movdqu.x [pgot], vd;
        movdqu.x [psel], vm;
    }
    if got[0] != 107 || got[1] != 9 || got[2] != 103 || got[3] != 103 fail(16, "pgatherdd");
    if sel[0] != 0 || sel[2] != 0 fail(17, "gather mask cleared");

    print("ok\n");
}
