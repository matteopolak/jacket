// AVX-512 #asm: op-mask (omr / k) registers and the k* instructions, compares into
// masks, `{k}` merge/zero masking, blendm, compress/expand, ternary logic,
// two-table permutes, mask <-> vector moves, test-into-mask, rotates,
// down-conversions, valign, scatter/gather with a mask, 128/256-bit chunk moves,
// conflict detection and AVX-512 conversions. No hardware here runs AVX-512, so
// the expectations are computed with ordinary Jai code next to each block.
#import "Basic";

failures := 0;

check :: (name: string, got: u64, want: u64) {
    if got != want {
        print("FAIL %: got 0x%, want 0x%\n", name, formatInt(got, base = 16), formatInt(want, base = 16));
        failures += 1;
    }
}

check_s32 :: (name: string, got: [] s32, want: [] s32) {
    for got if it != want[it_index] {
        print("FAIL % [%]: got %, want %\n", name, it_index, it, want[it_index]);
        failures += 1;
        return;
    }
}

main :: () {
    // k-register arithmetic.
    {
        a: u64 = 0b1100_1010_1111_0000;
        b: u64 = 0b1010_0110_0101_0101;
        r_and, r_or, r_xor, r_xnor, r_andn, r_not, r_add, r_shl, r_shr, r_unpck: u64;
        #asm AVX512F {
            kmovw ka:, a;
            kmovw kb:, b;
            kandw kr:, ka, kb;   kmovw r_and, kr;
            korw kr, ka, kb;     kmovw r_or, kr;
            kxorw kr, ka, kb;    kmovw r_xor, kr;
            kxnorw kr, ka, kb;   kmovw r_xnor, kr;
            kandnw kr, ka, kb;   kmovw r_andn, kr;
            knotw kr, ka;        kmovw r_not, kr;
            kaddw kr, ka, kb;    kmovw r_add, kr;
            kshiftlw kr, ka, 4;  kmovw r_shl, kr;
            kshiftrw kr, ka, 9;  kmovw r_shr, kr;
            kunpckbw kr, ka, kb; kmovw r_unpck, kr;
        }
        check("kandw", r_and, a & b);
        check("korw", r_or, a | b);
        check("kxorw", r_xor, a ^ b);
        check("kxnorw", r_xnor, ~(a ^ b) & 0xffff);
        check("kandnw", r_andn, ~a & b & 0xffff);
        check("knotw", r_not, ~a & 0xffff);
        check("kaddw", r_add, (a + b) & 0xffff);
        check("kshiftlw", r_shl, (a << 4) & 0xffff);
        check("kshiftrw", r_shr, a >> 9);
        check("kunpckbw", r_unpck, ((a & 0xff) << 8) | (b & 0xff));

        z, c: u8;
        zero: u64 = 0;
        ones: u64 = 0xffff;
        #asm AVX512F {
            kmovw k0:, zero;
            kmovw k1:, ones;
            kortestw k0, k0; setz z; setc c;
        }
        check("kortestw zero", (cast(u64) z) << 8 | c, 0x100);
        #asm AVX512F { kmovw k0:, zero; kmovw k1:, ones; kortestw k1, k0; setz z; setc c; }
        check("kortestw ones", (cast(u64) z) << 8 | c, 0x001);
        #asm AVX512F { kmovw ka:, a; kmovw kb:, b; ktestw ka, kb; setz z; setc c; }
        check("ktestw", (cast(u64) z) << 8 | c, 0);
        m8: u8 = 0xff;
        #asm AVX512DQ { kmovb kq:, m8; kmovq rq:, kq; }
        check("kmovb zero-extends", rq, 0xff);
    }

    X: [16] s32;
    Y: [16] s32;
    for 0..15 {
        X[it] = xx (it * 3 - 20);
        Y[it] = xx (10 - it * 2);
    }
    Y[5] = X[5];
    Y[12] = X[12];
    px, py := X.data, Y.data;
    out: [16] s32;
    po := out.data;
    want: [16] s32;

    // Compares into a mask.
    {
        eq, gt, ult, le_masked: u64;
        #asm AVX512F {
            movdqu32 zx:, [px];
            movdqu32 zy:, [py];
            pcmpeqd keq:, zx, zy;
            kmovw eq, keq;
            pcmpgtd kgt:, zx, zy;
            kmovw gt, kgt;
            pcmpud kult:, zx, zy, 1;
            kmovw ult, kult;
            pcmpd kle: &keq, zx, zy, 2;
            kmovw le_masked, kle;
        }
        e, g, u: u64;
        for 0..15 {
            if X[it] == Y[it] e |= (cast(u64) 1) << it;
            if X[it] > Y[it] g |= (cast(u64) 1) << it;
            if cast(u32) X[it] < cast(u32) Y[it] u |= (cast(u64) 1) << it;
        }
        check("pcmpeqd -> k", eq, e);
        check("pcmpgtd -> k", gt, g);
        check("pcmpud lt -> k", ult, u);
        check("pcmpd le & k", le_masked, e);
    }

    // {k} merging and zeroing.
    {
        mask: u64 = 0b0101_1010_0011_1100;
        #asm AVX512F {
            movdqu32 zx:, [px]; movdqu32 zy:, [py];
            kmovw km:, mask;
            movdqu32 acc:, [py];
            paddd acc &km, zx, zx;
            movdqu32 [po], acc;
        }
        for 0..15 want[it] = ifx mask & ((cast(u64) 1) << it) then X[it] * 2 else Y[it];
        check_s32("paddd merge", out, want);
        #asm AVX512F {
            movdqu32 zx:, [px]; movdqu32 zy:, [py];
            kmovw km:, mask;
            movdqu32 acc:, [py];
            paddd acc &*km, zx, zy;
            movdqu32 [po], acc;
        }
        for 0..15 want[it] = ifx mask & ((cast(u64) 1) << it) then X[it] + Y[it] else 0;
        check_s32("paddd zero", out, want);
        // A masked store only writes the selected lanes.
        for 0..15 out[it] = -1;
        #asm AVX512F { movdqu32 zx:, [px]; movdqu32 zy:, [py]; kmovw km:, mask; movdqu32 [po] &km, zx; }
        for 0..15 want[it] = ifx mask & ((cast(u64) 1) << it) then X[it] else -1;
        check_s32("masked store", out, want);
        #asm AVX512F { movdqu32 zx:, [px]; movdqu32 zy:, [py]; kmovw km:, mask; blendmps bl: &km, zx, zy; movdqu32 [po], bl; }
        for 0..15 want[it] = ifx mask & ((cast(u64) 1) << it) then Y[it] else X[it];
        check_s32("blendmps", out, want);
    }

    // Compress and expand.
    {
        mask: u64 = 0b1000_0100_0010_0101;
        selected: [..] s32;
        for 0..15 if mask & ((cast(u64) 1) << it) array_add(*selected, X[it]);
        for 0..15 out[it] = 77;
        #asm AVX512F {
            movdqu32 zx:, [px]; movdqu32 zy:, [py];
            kmovw kc:, mask;
            pcompressd [po] &kc, zx;
        }
        for 0..15 want[it] = ifx it < selected.count then selected[it] else 77;
        check_s32("pcompressd to memory", out, want);
        #asm AVX512F {
            movdqu32 zx:, [px]; movdqu32 zy:, [py];
            kmovw kc:, mask;
            compressps cz: &*kc, zx;
            movdqu32 [po], cz;
        }
        for 0..15 want[it] = ifx it < selected.count then selected[it] else 0;
        check_s32("compressps zeroing", out, want);
        #asm AVX512F {
            kmovw kc:, mask;
            pexpandd ez: &*kc, [py];
            movdqu32 [po], ez;
        }
        next := 0;
        for 0..15 {
            if mask & ((cast(u64) 1) << it) {
                want[it] = Y[next];
                next += 1;
            } else want[it] = 0;
        }
        check_s32("pexpandd", out, want);
    }

    // Ternary logic, two-table permutes, rotates.
    {
        #asm AVX512F {
            movdqu32 zx:, [px]; movdqu32 zy:, [py];
            movdqu32 t3:, zx;
            pternlogd t3, zy, zx, 0x96;
            movdqu32 [po], t3;
        }
        for 0..15 want[it] = X[it] ^ Y[it] ^ X[it];
        check_s32("pternlogd xor3", out, want);
        #asm AVX512F {
            movdqu32 zx:, [px]; movdqu32 zy:, [py];
            movdqu32 t3:, zx;
            pternlogd t3, zy, zy, 0xe8;
            movdqu32 [po], t3;
        }
        // jailint: allow(identical_operands) the majority of (x, y, y), one term per pair
        for 0..15 want[it] = (X[it] & Y[it]) | (X[it] & Y[it]) | (Y[it] & Y[it]);
        check_s32("pternlogd majority", out, want);

        idx: [16] s32;
        for 0..15 idx[it] = xx ((it * 5 + 3) & 31);
        pi := idx.data;
        #asm AVX512F {
            movdqu32 zx:, [px]; movdqu32 zy:, [py];
            movdqu32 zi:, [pi];
            movdqu32 t2:, zx;
            permt2d t2, zi, zy;
            movdqu32 [po], t2;
        }
        for 0..15 {
            k := idx[it];
            want[it] = ifx k < 16 then X[k] else Y[k - 16];
        }
        check_s32("permt2d", out, want);
        #asm AVX512F {
            movdqu32 zx:, [px]; movdqu32 zy:, [py];
            movdqu32 zi:, [pi];
            movdqu32 t2:, zi;
            permi2d t2, zx, zy;
            movdqu32 [po], t2;
        }
        check_s32("permi2d", out, want);
        #asm AVX512F { movdqu32 zx:, [px]; movdqu32 zy:, [py]; prold r:, zx, 7; movdqu32 [po], r; }
        for 0..15 {
            v := cast(u32) X[it];
            want[it] = xx ((v << 7) | (v >> 25));
        }
        check_s32("prold", out, want);
        #asm AVX512F { movdqu32 zx:, [px]; movdqu32 zy:, [py]; movdqu32 zi:, [pi]; prorvd r:, zx, zi; movdqu32 [po], r; }
        for 0..15 {
            v := cast(u32) X[it];
            n := cast(u32) (idx[it] & 31);
            want[it] = xx ifx n == 0 then v else (v >> n) | (v << (32 - n));
        }
        check_s32("prorvd", out, want);
    }

    // Mask <-> vector, test into mask, broadcast of a mask.
    {
        m: u64;
        mask: u64 = 0b0011_0000_1111_0001;
        #asm AVX512F, AVX512DQ {
            movdqu32 zx:, [px]; movdqu32 zy:, [py];
            kmovw kv:, mask;
            pmovm2d mv:, kv;
            movdqu32 [po], mv;
            pmovd2m kback:, zx;
            kmovw m, kback;
        }
        for 0..15 want[it] = ifx mask & ((cast(u64) 1) << it) then -1 else 0;
        check_s32("pmovm2d", out, want);
        s: u64;
        for 0..15 if X[it] < 0 s |= (cast(u64) 1) << it;
        check("pmovd2m", m, s);
        t, tn: u64;
        #asm AVX512F {
            movdqu32 zx:, [px]; movdqu32 zy:, [py];
            ptestmd ktm:, zx, zy;
            kmovw t, ktm;
            ptestnmd ktn:, zx, zy;
            kmovw tn, ktn;
        }
        e: u64;
        for 0..15 if X[it] & Y[it] e |= (cast(u64) 1) << it;
        check("ptestmd", t, e);
        check("ptestnmd", tn, ~e & 0xffff);
        #asm AVX512CD { kmovw kv:, mask; pbroadcastmw2d bm:, kv; movdqu32 [po], bm; }
        for 0..15 want[it] = xx mask;
        check_s32("pbroadcastmw2d", out, want);
    }

    // Down-conversions, valign, chunk moves.
    {
        bytes: [16] u8;
        words: [16] u16;
        pbytes, pwords := bytes.data, words.data;
        big: [16] s32;
        for 0..15 big[it] = xx ((it - 8) * 40);
        pbig := big.data;
        #asm AVX512F {
            movdqu32 zb:, [pbig];
            pmovdb [pbytes], zb;
        }
        for 0..15 if bytes[it] != cast(u8) (big[it] & 0xff) {
            check("pmovdb", xx it, 99);
            break;
        }
        #asm AVX512F { movdqu32 zb:, [pbig]; pmovsdb [pbytes], zb; }
        for 0..15 {
            v := clamp(big[it], -128, 127);
            if bytes[it] != cast(u8) (v & 0xff) {
                check("pmovsdb", xx it, 99);
                break;
            }
        }
        #asm AVX512F { movdqu32 zb:, [pbig]; pmovusdw [pwords], zb; }
        for 0..15 {
            v := cast(u32) big[it];
            w := ifx v > 0xffff then 0xffff else v;
            if words[it] != w {
                check("pmovusdw", xx it, 99);
                break;
            }
        }
        #asm AVX512F { movdqu32 zx:, [px]; movdqu32 zy:, [py]; valignd va:, zx, zy, 3; movdqu32 [po], va; }
        for 0..15 want[it] = ifx it + 3 < 16 then Y[it + 3] else X[it + 3 - 16];
        check_s32("valignd", out, want);
        #asm AVX512F { movdqu32 zx:, [px]; movdqu32 zy:, [py]; shufi32x4 sh:, zx, zy, 0b01_00_11_10; movdqu32 [po], sh; }
        chunks := int.[2, 3, 0, 1];
        for 0..15 {
            c := it / 4;
            src := ifx c < 2 then X else Y;
            want[it] = src[chunks[c] * 4 + it % 4];
        }
        check_s32("shufi32x4", out, want);
        #asm AVX512F {
            movdqu32 zx:, [px]; movdqu32 zy:, [py];
            extracti32x4 ex:, zy, 2;
            inserti32x4 ins:, zx, ex, 0;
            movdqu32 [po], ins;
        }
        for 0..15 want[it] = ifx it < 4 then Y[8 + it] else X[it];
        check_s32("extracti32x4/inserti32x4", out, want);
    }

    // Scatter and gather with a mask (EVEX forms clear the mask).
    {
        table: [32] s32;
        for 0..31 table[it] = xx (1000 + it);
        pt := table.data;
        idx: [16] s32;
        for 0..15 idx[it] = xx (31 - it * 2);
        pi := idx.data;
        mask: u64 = 0xf0f0;
        left: u64;
        #asm AVX512F {
            movdqu32 vi:, [pi];
            kmovw kg:, mask;
            movdqu32 g:, [py];
            pgatherdd g: &kg, [pt + vi*4];
            movdqu32 [po], g;
            kmovw left, kg;
        }
        for 0..15 want[it] = ifx mask & ((cast(u64) 1) << it) then table[idx[it]] else Y[it];
        check_s32("pgatherdd &k", out, want);
        check("pgatherdd clears k", left, 0);
        #asm AVX512F {
            movdqu32 zx:, [px]; movdqu32 zy:, [py];
            movdqu32 vi:, [pi];
            kmovw ks:, mask;
            pscatterdd [pt + vi*4] &ks, zx;
            kmovw left, ks;
        }
        for 0..15 if mask & ((cast(u64) 1) << it) && table[idx[it]] != X[it] {
            check("pscatterdd", xx it, 99);
            break;
        }
        check("pscatterdd untouched", xx table[31], 1031);
        check("pscatterdd clears k", left, 0);
    }

    // Bit counting and conflict detection.
    {
        dup: [16] s32 = .[5, 7, 5, 9, 7, 7, 1, 2, 3, 4, 5, 6, 0, 0, 0xffff, 1];
        pdup := dup.data;
        #asm AVX512CD {
            movdqu32 zd:, [pdup];
            pconflictd cf:, zd;
            movdqu32 [po], cf;
        }
        for i: 0..15 {
            m: s32;
            for j: 0..i - 1 if dup[j] == dup[i] m |= (cast(s32) 1) << j;
            want[i] = m;
        }
        check_s32("pconflictd", out, want);
        #asm AVX512CD { movdqu32 zd:, [pdup]; plzcntd lz:, zd; movdqu32 [po], lz; }
        for 0..15 {
            v := cast(u32) dup[it];
            n: s32 = 32;
            while v {
                v >>= 1;
                n -= 1;
            }
            want[it] = n;
        }
        check_s32("plzcntd", out, want);
        #asm AVX512_VPOPCNTDQ { movdqu32 zd:, [pdup]; popcntd pc:, zd; movdqu32 [po], pc; }
        for 0..15 {
            v := cast(u32) dup[it];
            n: s32;
            while v {
                n += xx (v & 1);
                v >>= 1;
            }
            want[it] = n;
        }
        check_s32("popcntd", out, want);
    }

    // Conversions and rounding.
    {
        f: [16] float32 = .[0.5, 1.5, 2.5, -0.5, 3e9, -1, 4294967295, 7.75, 0, 1, 2, 3, 4, 5, 6, 7];
        pf := f.data;
        u: [16] u32;
        pu := u.data;
        #asm AVX512F {
            movups fz:, [pf];
            cvtps2udq cu:, fz;
            movdqu32 [pu], cu;
        }
        expect_u := u32.[0, 2, 2, 0, 3_000_000_000, 0xffff_ffff, 0xffff_ffff, 8];
        for 0..7 if u[it] != expect_u[it] {
            check("cvtps2udq", xx it, u[it]);
            break;
        }
        q: [8] s64 = .[1, -2, 3, 1 << 53, -(1 << 40), 0, 7, -9];
        d: [8] float64;
        pq, pd := q.data, d.data;
        #asm AVX512DQ {
            movdqu64 zq:, [pq];
            cvtqq2pd zd2:, zq;
            movupd [pd], zd2;
        }
        for 0..7 if d[it] != cast(float64) q[it] {
            check("cvtqq2pd", xx it, 99);
            break;
        }
        r: [16] float32;
        pr := r.data;
        #asm AVX512F {
            movups fz:, [pf];
            rndscaleps rs:, fz, 0x10;
            movups [pr], rs;
        }
        check("rndscaleps halves", xx (r[7] == 8 && r[0] == 0.5 && r[3] == -0.5), 1);
        #asm AVX512F {
            movups fz:, [pf];
            rcp14ps rc:, fz;
            movups [pr], rc;
        }
        check("rcp14ps", xx (r[0] == 2 && r[4] < 3.4e-10 && r[5] == -1), 1);
    }

    // Chunk broadcasts and a non-temporal scalar store.
    {
        #asm AVX512F, AVX512DQ {
            broadcasti32x8 b8:, [px];
            movdqu32 [po], b8;
        }
        for 0..15 want[it] = X[it % 8];
        check_s32("broadcasti32x8", out, want);
        #asm AVX512DQ { broadcasti32x2 b2:, [py]; movdqu32 [po], b2; }
        for 0..15 want[it] = Y[it % 2];
        check_s32("broadcasti32x2", out, want);
        nt: u64;
        value: u64 = 0x1234_5678_9abc_def0;
        pnt := *nt;
        #asm { movnti [pnt], value; }
        check("movnti", nt, value);
    }

    if failures exit(1);
    print("ok\n");
}
