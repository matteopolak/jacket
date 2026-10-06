// Checks the rules of docs/language/simd-asm.md. #asm is lowered to IR, so this runs on any host.
// rules: asm.1 asm.2 asm.3 asm.4 asm.5 asm.6 asm.7 asm.8 asm.9 asm.10 asm.11 asm.12 asm.13 asm.15 asm.16 asm.17 asm.18 asm.19 asm.20 asm.21 asm.24 asm.25
#import "Basic";

main :: () {
    // asm.1: variables are read and written in place; immediates and memory operands work.
    {
        total: s64 = 1;
        step: s64 = 6;
        cells: [3] s32 = .[10, 20, 30];
        cp := cells.data;
        i: s64 = 2;
        #asm {
            add total, step;
            add total, 3;
            mov.d [cp + i*4], 77;
        }
        assert(total == 10 && cells[2] == 77 && cells[1] == 20, "asm.1");
    }

    // asm.2: pinning a variable to a register is accepted and changes nothing.
    {
        v: u64 = 5;
        #asm {
            v === c;
            add v, 2;
        }
        assert(v == 7, "asm.2");
    }

    // asm.3: a declared gpr starts at zero and outlives its block.
    {
        #asm {
            scratch: gpr;
            add scratch, 9;
        }
        out: s64;
        #asm { mov out, scratch; }
        assert(out == 9, "asm.3");
    }

    // asm.4: the documented example.
    {
        x: s64 = 10;
        #asm { add x, 5; sub x, 3; }
        assert(x == 12, "asm.4");
    }

    // asm.5: the documented 128-bit multiply.
    {
        a := float32.[1, 2, 3, 4];
        b := float32.[10, 20, 30, 40];
        pa, pb := a.data, b.data;
        #asm {
            movups.x v:, [pa];
            movups.x w:, [pb];
            mulps.x v, w;
            movups.x [pa], v;
        }
        assert(a[0] == 10 && a[1] == 40 && a[2] == 90 && a[3] == 160, "asm.5");
    }

    // asm.6: size from the suffix, else the first variable operand, else 64 bits.
    {
        small: u8 = 255;
        carried: bool;
        #asm {
            add small, 1;
            setc carried;
        }
        assert(small == 0 && carried, "asm.6 variable size");

        wide: u64 = 0xff;
        #asm {
            add.b wide, 1;
            setc carried;
        }
        assert(wide == 0 && carried, "asm.6 suffix");

        top: u64;
        #asm {
            mov tmp:, 0xffff_ffff;
            add tmp, 1;
            setc carried;
            mov top, tmp;
        }
        assert(top == 0x1_0000_0000 && !carried, "asm.6 default 64 bits");
    }

    // asm.7, asm.8: 32-bit writes zero-extend, 8/16-bit writes merge.
    {
        y: u64 = 0xaaaa_bbbb_cccc_dddd;
        #asm { mov.d y, 1; }
        assert(y == 1, "asm.7");
        z: u64 = 0xaaaa_bbbb_cccc_dddd;
        #asm { mov.w z, 1; }
        assert(z == 0xaaaa_bbbb_cccc_0001, "asm.8 word");
        q: u64 = 0xaaaa_bbbb_cccc_dddd;
        #asm { mov.b q, 1; }
        assert(q == 0xaaaa_bbbb_cccc_dd01, "asm.8 byte");
    }

    // asm.9, asm.10, asm.11: an AVX block works on 256 bits; vaddps is accepted; three operands.
    {
        a: [8] float32 = .[1, 2, 3, 4, 5, 6, 7, 8];
        b: [8] float32 = .[10, 10, 10, 10, 10, 10, 10, 10];
        r: [8] float32;
        pa, pb, pr := a.data, b.data, r.data;
        #asm AVX {
            movups va:, [pa];
            movups vb:, [pb];
            vaddps vr:, va, vb;
            movups [pr], vr;
        }
        for r assert(it == a[it_index] + 10, "asm.9 asm.10 asm.11");
        assert(a[0] == 1 && b[0] == 10, "asm.11 sources unchanged");
    }

    // asm.12: setcc and cmovcc read the flags of an earlier instruction in the block.
    {
        lo: s64 = 3;
        hi: s64 = 8;
        less: bool;
        pick: s64 = 0;
        #asm {
            cmp lo, hi;
            setl less;
            cmovl pick, hi;
        }
        assert(less && pick == 8, "asm.12");
    }

    // asm.13: lock-prefixed memory operations update memory atomically.
    {
        counter: s64 = 40;
        p := *counter;
        #asm {
            mov add_by:, 2;
            lock_xadd.q [p], add_by;
        }
        assert(counter == 42, "asm.13");
    }

    // asm.15: implicit registers are explicit operands.
    {
        hi: u64 = 0;
        lo: u64 = 47;
        d: u64 = 5;
        #asm { div hi, lo, d; }
        assert(lo == 9 && hi == 2, "asm.15 div");

        src: [4] u64 = .[1, 2, 3, 4];
        dst: [4] u64;
        di, si := dst.data, src.data;
        n: u64 = 4;
        #asm { rep_movs.q di, si, n; }
        assert(dst[3] == 4 && n == 0, "asm.15 rep_movs");

        x: [3] u8 = .[1, 2, 3];
        y: [3] u8 = .[1, 2, 9];
        px, py := x.data, y.data;
        cnt: u64 = 3;
        same: bool;
        #asm {
            repe_cmps.b px, py, cnt;
            setz same;
        }
        assert(!same && cnt == 0, "asm.15 repe_cmps");

        neg: s64 = -5;
        sign: s64;
        #asm { cqo sign, neg; }
        assert(sign == -1, "asm.15 cqo");

        pair: [2] u64 #align 16 = .[1, 2];
        pp := pair.data;
        exp_lo: u64 = 1;
        exp_hi: u64 = 2;
        new_lo: u64 = 7;
        new_hi: u64 = 8;
        swapped: bool;
        #asm {
            cmpxchg16b exp_hi, exp_lo, [pp], new_hi, new_lo;
            setz swapped;
        }
        assert(swapped && pair[0] == 7 && pair[1] == 8, "asm.15 cmpxchg16b");
    }

    // asm.16: a feature modifier is accepted on any host and VEX forms zero the upper bytes.
    {
        a: [8] float32 = .[1, 1, 1, 1, 1, 1, 1, 1];
        r: [8] float32;
        pa, pr := a.data, r.data;
        #asm AVX2, FMA, BMI2 {
            movups wide:, [pa];
            addps.x wide, wide;
            movups [pr], wide;
        }
        assert(r[0] == 2 && r[3] == 2 && r[4] == 0 && r[7] == 0, "asm.16 VEX zeroing");
        r2: [8] float32;
        p2 := r2.data;
        #asm {
            movups.y keep:, [pa];
            addps.x keep, keep;
            movups.y [p2], keep;
        }
        assert(r2[0] == 2 && r2[4] == 1, "asm.16 legacy keeps upper bytes");
    }

    // asm.17, asm.18: broadcast and rounding decorations.
    {
        out: [16] s32;
        po := out.data;
        f: float32 = 2.5;
        pf := *f;
        #asm AVX512F {
            cvtps2dq near:, [pf]!;
            movdqu [po], near;
        }
        for out assert(it == 2, "asm.17 broadcast, round to even");
        #asm AVX512F {
            broadcastss src:, [pf];
            cvtps2dq up:, src !u;
            movdqu [po], up;
        }
        for out assert(it == 3, "asm.18 round up");
        #asm AVX512F {
            broadcastss src2:, [pf];
            cvtps2dq down:, src2 !d;
            movdqu [po], down;
        }
        for out assert(it == 2, "asm.18 round down");
    }

    // asm.19, asm.21: a compare writes a mask; zero and merge masking.
    {
        a: [16] s32;
        b: [16] s32;
        for 0..15 {
            a[it] = xx it;
            b[it] = 8;
        }
        zeroed: [16] s32;
        merged: [16] s32;
        bits: u64;
        pa, pb, pz, pm := a.data, b.data, zeroed.data, merged.data;
        #asm AVX512F {
            movdqu va:, [pa];
            movdqu vb:, [pb];
            pcmpgtd kgt:, va, vb;
            kmovw bits, kgt;
            movdqu vz:, [pb];
            paddd vz &* kgt, va, va;
            movdqu [pz], vz;
            movdqu vm:, [pb];
            paddd vm & kgt, va, va;
            movdqu [pm], vm;
        }
        assert(bits == 0xfe00, "asm.21 compare into mask");
        for 0..15 {
            if it > 8 {
                assert(zeroed[it] == it * 2 && merged[it] == it * 2, "asm.19 selected lanes");
            } else {
                assert(zeroed[it] == 0, "asm.19 zero masking");
                assert(merged[it] == 8, "asm.19 merge masking");
            }
        }
    }

    // asm.20: a masked store writes only the selected elements.
    {
        dst: [16] s32;
        for *dst it.* = -1;
        pd := dst.data;
        sel: u64 = 0b11;
        #asm AVX512F {
            kmovw ks:, sel;
            mov seven:, 7;
            pbroadcastd vs:, seven;
            movdqu32 [pd] & ks, vs;
        }
        assert(dst[0] == 7 && dst[1] == 7 && dst[2] == -1 && dst[15] == -1, "asm.20");
    }

    // asm.24: rcp and rsqrt are exact.
    {
        v: [4] float32 = .[3, 4, 16, 0.5];
        rc: [4] float32;
        rs: [4] float32;
        pv, prc, prs := v.data, rc.data, rs.data;
        #asm {
            movups.x x:, [pv];
            rcpps.x y:, x;
            rsqrtps.x z:, x;
            movups.x [prc], y;
            movups.x [prs], z;
        }
        assert(rc[0] == 1.0 / 3.0 && rc[1] == 0.25 && rc[3] == 2, "asm.24 rcp");
        assert(rs[1] == 0.5 && rs[2] == 0.25, "asm.24 rsqrt");
    }

    // asm.25: cpuid and xgetbv report no features.
    {
        fb, fc, fd, xa: u32;
        #asm {
            mov a:, 1;
            mov c:, 0;
            cpuid a, b:, c, d:;
            mov fb, b;
            mov fc, c;
            mov fd, d;
            mov xc:, 0;
            xgetbv xd:, xa2:, xc;
            mov xa, xa2;
        }
        assert(fb == 0 && fc == 0 && fd == 0 && xa == 0, "asm.25");
    }

    print("ok\n");
}
