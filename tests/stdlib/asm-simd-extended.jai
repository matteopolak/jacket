// SSE..AVX2, FMA, AES and PCLMULQDQ #asm instructions beyond the core vector set:
// saturating and multiply-high arithmetic, horizontal ops, unpacks, shuffles,
// permutes, blends, inserts/extracts, sign/zero extension, packs, conversions,
// compares, rounding, FMA, variable shifts, dot products, AES rounds and
// carry-less multiplies. Each result is hashed (FNV-1a over its bytes); the
// expected hashes come from the same instruction sequences on x86-64 hardware.
#import "Basic";

failures := 0;

hash :: (p: *u8, n: int) -> u64 {
    h: u64 = 0xcbf2_9ce4_8422_2325;
    for 0..n - 1 {
        h ^= p[it];
        h *= 0x100_0000_01b3;
    }
    return h;
}

check :: (name: string, got: u64, want: u64) {
    if got != want {
        print("FAIL %: got 0x%, want 0x%\n", name, formatInt(got, base = 16), formatInt(want, base = 16));
        failures += 1;
    }
}

main :: () {
    A: [64] u8;
    B: [64] u8;
    for 0..63 {
        A[it] = xx ((it * 7 + 3) & 0xff);
        B[it] = xx ((200 - it * 13) & 0xff);
    }
    F: [16] float32;
    G: [16] float32;
    for 0..15 {
        F[it] = (it - 5.25) * 1.5;
        G[it] = 3.5 - it * 0.75;
    }
    D: [8] float64;
    E: [8] float64;
    for 0..7 {
        D[it] = (it - 2.5) * 3.25;
        E[it] = 1.75 - it * 1.125;
    }
    out: [64] u8;
    pa, pb, pf, pg, pd, pe, po := A.data, B.data, F.data, G.data, D.data, E.data, out.data;

    h :: (n: int) -> u64 #expand {
        return hash(`po, n);
    }
    clear :: () #expand {
        memset(`po, 0, 64);
    }

    // Saturating arithmetic, averages, multiply-high, psign (legacy SSE forms).
    #asm {
        movdqu.x a:, [pa];
        movdqu.x b:, [pb];
        movdqu.x r:, a;
        paddsb.x r, b;
        movdqu.x [po], r;
    }
    check("paddsb", h(16), 0xf92579310df573);
    #asm { movdqu.x r, a; paddusw.x r, b; movdqu.x [po], r; }
    check("paddusw", h(16), 0xbedecd405a59c245);
    #asm { movdqu.x r, a; psubsw.x r, b; movdqu.x [po], r; }
    check("psubsw", h(16), 0x3a70e477387a1d61);
    #asm { movdqu.x r, a; psubusb.x r, b; movdqu.x [po], r; }
    check("psubusb", h(16), 0x5268cb49205ba8c3);
    #asm { movdqu.x r, a; pavgb.x r, b; movdqu.x [po], r; }
    check("pavgb", h(16), 0xae8bf100787eb25);
    #asm { movdqu.x r, a; pmulhw.x r, b; movdqu.x [po], r; }
    check("pmulhw", h(16), 0x74dc31dd31d3ad8c);
    #asm { movdqu.x r, a; pmulhuw.x r, b; movdqu.x [po], r; }
    check("pmulhuw", h(16), 0xb82dbc91672bed95);
    #asm { movdqu.x r, a; pmulhrsw.x r, b; movdqu.x [po], r; }
    check("pmulhrsw", h(16), 0x812e333b0e1915ec);
    #asm { movdqu.x r, a; psignb.x r, b; movdqu.x [po], r; }
    check("psignb", h(16), 0x8c6bf72b3e669023);
    #asm { movdqu.x r, a; pmaddwd.x r, b; movdqu.x [po], r; }
    check("pmaddwd", h(16), 0xc7c4c8f7f3e4fe8d);
    #asm { movdqu.x r, a; pmaddubsw.x r, b; movdqu.x [po], r; }
    check("pmaddubsw", h(16), 0x97c3554a0109e38);
    #asm { movdqu.x r, a; psadbw.x r, b; movdqu.x [po], r; }
    check("psadbw", h(16), 0x841945924454de73);
    #asm { movdqu.x r, a; phaddw.x r, b; movdqu.x [po], r; }
    check("phaddw", h(16), 0xf44a8a5754a26e49);
    #asm { movdqu.x r, a; phsubsw.x r, b; movdqu.x [po], r; }
    check("phsubsw", h(16), 0xb3adde70fb6496d5);
    #asm { movdqu.x r, a; phminposuw r, b; movdqu.x [po], r; }
    check("phminposuw", h(16), 0x769e0a97b6acdf3f);

    // MMX (`str` registers, .q).
    #asm {
        movq m: str, [pa];
        movq n: str, [pb];
        paddsw.q m, n;
        movq [po], m;
    }
    check("paddsw mmx", h(8), 0xdf977c2e5f9030f3);

    // AVX2 three-operand forms.
    #asm AVX2 {
        movdqu ya:, [pa];
        movdqu yb:, [pb];
        pmuludq yr:, ya, yb;
        movdqu [po], yr;
    }
    check("pmuludq", h(32), 0xd40d8fe213707e9f);
    #asm AVX2 { pmuldq yr, ya, yb; movdqu [po], yr; }
    check("pmuldq", h(32), 0xe9f318424596faf9);
    #asm AVX2 { phaddd yr, ya, yb; movdqu [po], yr; }
    check("phaddd", h(32), 0x5aa590cedc22945);
    #asm AVX2 { punpckhwd yr, ya, yb; movdqu [po], yr; }
    check("punpckhwd", h(32), 0x9aa2d465f3746e85);
    #asm AVX2 { punpcklbw.x yr, ya, yb; movdqu.x [po], yr; }
    check("punpcklbw", h(16), 0x1c885bf22dfc17cd);
    #asm AVX2 { punpcklqdq.x yr, ya, yb; movdqu.x [po], yr; }
    check("punpcklqdq", h(16), 0xbe29c5a8f507c0d5);
    #asm AVX2 { pshufhw.x yr, ya, 0x1b; movdqu.x [po], yr; }
    check("pshufhw", h(16), 0x2dd6cbb326eceb85);
    #asm AVX2 { pshuflw.x yr, ya, 0x72; movdqu.x [po], yr; }
    check("pshuflw", h(16), 0xcb71870cf10e1cd1);
    #asm AVX2 { palignr yr, ya, yb, 5; movdqu [po], yr; }
    check("palignr", h(32), 0xf84c58c6802b2835);
    #asm { movdqu.x r, a; palignr.x r, b, 17; movdqu.x [po], r; }
    check("palignr legacy", h(16), 0x7f7a811b25ddc20c);
    #asm AVX2 { pslldq yr, ya, 3; movdqu [po], yr; }
    check("pslldq", h(32), 0xdb3e450f954ef201);
    #asm AVX2 { psrldq yr, ya, 5; movdqu [po], yr; }
    check("psrldq", h(32), 0xcabd32ede6ed8611);
    #asm AVX2 { pblendw yr, ya, yb, 0xa5; movdqu [po], yr; }
    check("pblendw", h(32), 0x7ee97f00555650a5);
    #asm AVX2 { pblendvb yr, ya, yb, yb; movdqu [po], yr; }
    check("pblendvb", h(32), 0x922fa2c9fbfabb9);
    #asm AVX2 { pblendd yr, ya, yb, 0x5a; movdqu [po], yr; }
    check("pblendd", h(32), 0xe003e59c05c5cc5);
    #asm AVX2 { pshufb yr, ya, yb; movdqu [po], yr; }
    check("pshufb", h(32), 0x5b16562b679cbdc5);
    #asm AVX2 { permd yr, yb, ya; movdqu [po], yr; }
    check("permd", h(32), 0xc6f460c8f5fbf785);
    #asm AVX2 { permq yr, ya, 0x4e; movdqu [po], yr; }
    check("permq", h(32), 0x53bd2c007c421e25);
    #asm AVX2 { perm2i128 yr, ya, yb, 0x21; movdqu [po], yr; }
    check("perm2i128", h(32), 0x7f7c14923cd2b845);
    #asm AVX2 { perm2i128 yr, ya, yb, 0x83; movdqu [po], yr; }
    check("perm2i128 zero", h(32), 0xa8a1725ff04f9305);
    #asm AVX2 { inserti128 yr, ya, yb, 1; movdqu [po], yr; }
    check("inserti128", h(32), 0xf400c2945cf91345);
    clear();
    #asm AVX2 { extracti128 yr, yb, 1; movdqu [po], yr; }
    check("extracti128", h(32), 0xa8a1725ff04f9305);
    #asm AVX2 { pmovsxbw yr, ya; movdqu [po], yr; }
    check("pmovsxbw", h(32), 0x4125c366c659c785);
    #asm AVX2 { pmovzxwd.x yr, ya; movdqu.x [po], yr; }
    check("pmovzxwd", h(16), 0xde6d8e8d713885e5);
    #asm AVX2 { pmovsxdq yr, yb; movdqu [po], yr; }
    check("pmovsxdq", h(32), 0xfc17ff79c976d379);
    #asm AVX2 { packsswb.x yr, ya, yb; movdqu.x [po], yr; }
    check("packsswb", h(16), 0x25c2fac9ca638f1c);
    #asm AVX2 { packuswb yr, ya, yb; movdqu [po], yr; }
    check("packuswb", h(32), 0xbf3108ca0db50028);
    #asm AVX2 { packssdw.x yr, ya, yb; movdqu.x [po], yr; }
    check("packssdw", h(16), 0xf98a959fd42f510f);
    #asm AVX2 { packusdw.x yr, ya, yb; movdqu.x [po], yr; }
    check("packusdw", h(16), 0xf65a0eddc1aeb90f);
    #asm AVX2 { psllvd yr, ya, yb; movdqu [po], yr; }
    check("psllvd", h(32), 0xc8210784d8af5a5);
    #asm AVX2 {
        pxor cnt:, cnt, cnt;
        movq.x cnt, [pa];
        pmovzxbq cnt, cnt;
        psrlvq yr, yb, cnt;
        movdqu [po], yr;
    }
    check("psrlvq", h(32), 0x74b6fa51cd1d3f16);
    #asm AVX2 { pmovzxbd.x cnt, [pa]; psravd.x yr, yb, cnt; movdqu.x [po], yr; }
    check("psravd", h(16), 0x88bd141608478c9e);

    // Scalar inserts and extracts.
    {
        x: u64 = 0x1122_3344_5566_7788;
        r16: u64 = 0xffff_ffff_ffff_ffff;
        r64: u64;
        #asm {
            movdqu.x v:, [pa];
            pinsrb v, x, 5;
            pinsrd v, x, 3;
            movdqu.x [po], v;
            pextrw r16, v, 2;
            pextrq r64, v, 1;
        }
        check("pinsrb/pinsrd", h(16), 0xe208ed6a92c8132b);
        check("pextrw", r16, 0x881f);
        check("pextrq", r64, 0x556677885049423b);
    }

    // Float shuffles and moves.
    #asm AVX2 {
        movups fa:, [pf];
        movups fb:, [pg];
        unpcklps.x fr:, fa, fb;
        movups.x [po], fr;
    }
    check("unpcklps", h(16), 0xeb7f46b66c6a9a65);
    #asm AVX2 { unpckhpd fr, fa, fb; movups [po], fr; }
    check("unpckhpd", h(32), 0xad2bf10baca2c9c9);
    #asm AVX2 { shufpd fr, fa, fb, 0x9; movups [po], fr; }
    check("shufpd", h(32), 0x37459078f3e5a70e);
    #asm AVX2 { blendps fr, fa, fb, 0x5a; movups [po], fr; }
    check("blendps", h(32), 0xcde1f4f6fe9e96aa);
    #asm { movups.x x:, [pf]; movups.x y:, [pg]; movups.x z:, [pb]; blendvps.x x, y, z; movups.x [po], x; }
    check("blendvps legacy", h(16), 0xbc5c2ad2120c989d);
    #asm AVX2 { permilps.x fr, fa, 0x93; movups.x [po], fr; }
    check("permilps imm", h(16), 0x1c5025b046d4ef1);
    #asm AVX2 { movdqu ctl:, [pb]; permilps fr, fa, ctl; movups [po], fr; }
    check("permilps var", h(32), 0xf1e9328367d93b45);
    #asm AVX2 { permilpd fr, fa, 0x6; movups [po], fr; }
    check("permilpd imm", h(32), 0xa0e12f208a15ac7);
    #asm AVX2 { permps fr, ctl, fa; movups [po], fr; }
    check("permps", h(32), 0xd98f8a0efce8c05);
    #asm AVX2 { insertps fr, fa, fb, 0x9a; movups.x [po], fr; }
    check("insertps", h(16), 0x1b3046fd26971685);
    {
        e: u32;
        #asm AVX2 { extractps e, fb, 2; }
        check("extractps", e, 0x40000000);
    }
    #asm AVX2 { movhlps fr, fa, fb; movups.x [po], fr; }
    check("movhlps", h(16), 0xca2d043ea6b4e5cc);
    #asm AVX2 { movlhps fr, fa, fb; movups.x [po], fr; }
    check("movlhps", h(16), 0xd526893a7528bb25);
    #asm AVX2 { movhps fr, fa, [pg]; movups.x [po], fr; }
    check("movhps load", h(16), 0xd526893a7528bb25);
    clear();
    #asm AVX2 { movhps [po], fb; }
    check("movhps store", h(16), 0x549c7303db284128);
    #asm AVX2 { movddup fr, fa; movups [po], fr; }
    check("movddup", h(32), 0x5aab5bb3d1f61805);
    #asm AVX2 { movshdup.x fr, fa; movups.x [po], fr; }
    check("movshdup", h(16), 0x6b6f8cae7aa4b945);
    #asm AVX2 { movsldup.x fr, fb; movups.x [po], fr; }
    check("movsldup", h(16), 0x88f604e6a57b5665);
    #asm AVX2 { haddps.x fr, fa, fb; movups.x [po], fr; }
    check("haddps", h(16), 0xa85bcdb1d425f1a5);
    #asm AVX2 { movupd da:, [pd]; movupd db:, [pe]; hsubpd dr:, da, db; movupd [po], dr; }
    check("hsubpd", h(32), 0x9002e715678027f5);
    #asm AVX2 { addsubps fr, fa, fb; movups [po], fr; }
    check("addsubps", h(32), 0x5e996e129cdc477c);
    #asm AVX2 { dpps fr, fa, fb, 0xf1; movups [po], fr; }
    check("dpps", h(32), 0xafd0c2c3a744171);
    #asm AVX2 { dppd.x dr, da, db, 0x31; movupd.x [po], dr; }
    check("dppd", h(16), 0x4b170bf48b1c4c0);

    // Conversions.
    #asm AVX2 { cvtdq2pd dr, ya; movupd [po], dr; }
    check("cvtdq2pd", h(32), 0x28d05feace2e214f);
    clear();
    #asm AVX2 { cvtpd2dq dr, da; movdqu.x [po], dr; }
    check("cvtpd2dq", h(16), 0x18959266cc757c71);
    clear();
    #asm AVX2 { cvttpd2dq.x dr, db; movdqu.x [po], dr; }
    check("cvttpd2dq", h(16), 0x392209f14dea4c24);
    #asm AVX2 { cvtps2pd dr, fa; movupd [po], dr; }
    check("cvtps2pd", h(32), 0x64a26603fa9a303b);
    clear();
    #asm AVX2 { cvtpd2ps dr, db; movups.x [po], dr; }
    check("cvtpd2ps", h(16), 0x69facc7a0b2d97b5);
    #asm AVX2 { cvtss2sd dr, da, fa; movupd.x [po], dr; }
    check("cvtss2sd", h(16), 0xd171b1cdcf96b799);
    #asm AVX2 { cvtsd2ss fr, fa, db; movups.x [po], fr; }
    check("cvtsd2ss", h(16), 0xab706c1fb755ab0);
    {
        q: s64 = -123456789012;
        d: s32 = -77;
        i32: s32;
        i64: s64;
        #asm AVX2 {
            cvtsi2ss.q fr, fa, q;
            movups.x [po], fr;
        }
        check("cvtsi2ss.q", h(16), 0xaa263db0f70fd552);
        #asm AVX2 { cvtsi2sd.d dr, da, d; movupd.x [po], dr; }
        check("cvtsi2sd.d", h(16), 0xa383b1cd7918c685);
        #asm AVX2 { cvtss2si.d i32, fa; cvttsd2si.q i64, db; }
        check("cvtss2si.d", cast(u32) i32, 0xfffffff8);
        check("cvttsd2si.q", cast(u64) i64, 0x1);
    }

    // Compares, NaNs and flags.
    {
        nan: [8] float32 = .[1, 2, 0, 0, 5, 6, 7, 8];
        quiet_nan: u32 = 0x7fc0_0000;
        nan[2] = (cast(*float32) *quiet_nan).*;
        pn := nan.data;
        #asm AVX2 {
            movups fn:, [pn];
            cmpps fr, fn, fa, 1;
            movups [po], fr;
        }
        check("cmpps lt", h(32), 0xc8210784d8af5a5);
        #asm AVX2 { cmpps fr, fn, fa, 3; movups [po], fr; }
        check("cmpps unord", h(32), 0x6f8a4bd59751c7c1);
        #asm AVX2 { cmpps fr, fn, fa, 4; movups [po], fr; }
        check("cmpps neq", h(32), 0x9ac5e12119fd3f85);
        #asm AVX2 { cmpps fr, fn, fa, 14; movups [po], fr; }
        check("cmpps gt", h(32), 0x8f646f2069e12b69);
        #asm AVX2 { cmpsd dr, da, db, 2; movupd.x [po], dr; }
        check("cmpsd le", h(16), 0x231d9e33f53fd336);
        z, c, p: u8;
        #asm AVX2 { comiss fa, fb; setz z; setc c; setp p; }
        check("comiss", ((cast(u64) z) << 16) | ((cast(u64) c) << 8) | p, 0x100);
        #asm AVX2 { movups.x fu:, [pn + 8]; ucomiss fu, fa; setz z; setc c; setp p; }
        check("ucomiss unordered", ((cast(u64) z) << 16) | ((cast(u64) c) << 8) | p, 0x10101);
        #asm AVX2 { ptest ya, yb; setz z; setc c; }
        check("ptest", (cast(u64) z) << 8 | c, 0x0);
        #asm AVX2 { pxor zz:, zz, zz; ptest zz, ya; setz z; setc c; }
        check("ptest zero", (cast(u64) z) << 8 | c, 0x100);
    }

    // Rounding.
    #asm AVX2 { roundps fr, fa, 0; movups [po], fr; }
    check("roundps nearest", h(32), 0x8eb772698734569b);
    #asm AVX2 { roundps fr, fa, 1; movups [po], fr; }
    check("roundps down", h(32), 0x889c2096c50b2cd2);
    #asm AVX2 { roundps fr, fa, 2; movups [po], fr; }
    check("roundps up", h(32), 0x59dff49ff3e544f8);
    #asm AVX2 { roundps fr, fa, 3; movups [po], fr; }
    check("roundps trunc", h(32), 0x61f00ff5fbdde665);
    #asm AVX2 { roundsd dr, da, db, 0; movupd.x [po], dr; }
    check("roundsd", h(16), 0xb73f3d20fb8fa8fe);

    // FMA.
    #asm AVX2, FMA {
        movups fr, [pf];
        fmadd132ps fr, fa, fb;
        movups [po], fr;
    }
    check("fmadd132ps", h(32), 0xecd9250b59b16e2f);
    #asm AVX2, FMA { movups fr, fb; fmadd231ps.x fr, fa, fa; movups.x [po], fr; }
    check("fmadd231ps", h(16), 0xe2d24aa5e93e6b47);
    #asm AVX2, FMA { movupd dr, da; fmadd213pd dr, db, da; movupd [po], dr; }
    check("fmadd213pd", h(32), 0xba7d4a18ef489806);
    #asm AVX2, FMA { movupd dr, da; fnmadd231sd dr, db, db; movupd.x [po], dr; }
    check("fnmadd231sd", h(16), 0xce3515bbec25af88);
    #asm AVX2, FMA { movups fr, fa; fmsub213ps fr, fb, fa; movups [po], fr; }
    check("fmsub213ps", h(32), 0x45bb731d5ad4501f);
    #asm AVX2, FMA { movups fr, fa; fmaddsub231ps fr, fb, fa; movups [po], fr; }
    check("fmaddsub231ps", h(32), 0x94735abf78a3e75e);
    #asm AVX2, FMA { movupd dr, db; fmsubadd132pd dr, da, db; movupd [po], dr; }
    check("fmsubadd132pd", h(32), 0x1e47a1dc01178cfa);
    #asm AVX2, FMA { movupd dr, db; fnmsub213pd dr, da, db; movupd [po], dr; }
    check("fnmsub213pd", h(32), 0x870f9a969282a42);

    // AES and carry-less multiplication.
    #asm AES {
        movdqu.x s:, [pa];
        movdqu.x k:, [pb];
        movdqu.x t:, s;
        aesenc t, k;
        movdqu.x [po], t;
    }
    check("aesenc", h(16), 0x6c1632d0a2b475fd);
    #asm AES { movdqu.x t, s; aesenclast t, k; movdqu.x [po], t; }
    check("aesenclast", h(16), 0x791659a4b6c250e5);
    #asm AES { movdqu.x t, s; aesdec t, k; movdqu.x [po], t; }
    check("aesdec", h(16), 0x4b67a030f585f686);
    #asm AES { movdqu.x t, s; aesdeclast t, k; movdqu.x [po], t; }
    check("aesdeclast", h(16), 0x6ff86c272aae4d3c);
    #asm AES { aesimc t, s; movdqu.x [po], t; }
    check("aesimc", h(16), 0xf157a55b3a91b981);
    #asm AES { aeskeygenassist t, s, 0x1b; movdqu.x [po], t; }
    check("aeskeygenassist", h(16), 0x880b31e91237a15f);
    #asm PCLMULQDQ { movdqu.x t, s; pclmulqdq t, k, 0x00; movdqu.x [po], t; }
    check("pclmulqdq 00", h(16), 0x9cf606846d4f41f0);
    #asm PCLMULQDQ { movdqu.x t, s; pclmulqdq t, k, 0x11; movdqu.x [po], t; }
    check("pclmulqdq 11", h(16), 0xedbc581aa82981e6);
    #asm PCLMULQDQ { movdqu.x t, s; pclmulhqlqdq t, k; movdqu.x [po], t; }
    check("pclmulhqlqdq", h(16), 0x26240b99917a76b0);

    // Masked moves with a vector mask.
    clear();
    #asm AVX2 { pmaskmovd yr, yb, [pa]; movdqu [po], yr; }
    check("pmaskmovd load", h(32), 0x9789ed6cbd143fd9);
    clear();
    #asm AVX2 { maskmovps [po], ya, fb; }
    check("maskmovps store", h(32), 0xae4a8c08f4b84d2);

    if failures exit(1);
    print("ok\n");
}
