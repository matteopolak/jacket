// General-purpose #asm instructions beyond the core ALU set: division, string
// instructions, accumulator widening, double and through-carry shifts, BMI1/BMI2,
// ADX, CRC32, parity, lahf/sahf, xlat and cmpxchg16b. The expected values were
// produced by the same instruction sequences on x86-64 hardware.
#import "Basic";

failures := 0;

check :: (name: string, got: u64, want: u64) {
    if got != want {
        print("FAIL %: got 0x%, want 0x%\n", name, formatInt(got, base = 16), formatInt(want, base = 16));
        failures += 1;
    }
}

main :: () {
    // div / idiv: `div hi, lo, divisor` divides hi:lo; the quotient goes to lo, the remainder to hi.
    {
        hi: u64 = 1;
        lo: u64 = 0x23456789abcdef01;
        d: u64 = 0x3456789abcd;
        #asm { div hi, lo, d; }
        check("div.q quotient", lo, 0x590b21);
        check("div.q remainder", hi, 0x147ae67fa94);
    }
    {
        hi: u64 = 0xffff_ffff_ffff_fffe;
        lo: u64 = 0x1234;
        d: s64 = 1000;
        #asm { idiv hi, lo, d; }
        check("idiv.q quotient", lo, 0xff7ced916872b026);
        check("idiv.q remainder", hi, 0xfffffffffffffdc4);
    }
    {
        hi: u64 = 0xffff_ffff_ffff_ffff;
        lo: u64 = 0x8000_0000_0000_0001;
        d: s64 = -3;
        #asm { idiv hi, lo, d; }
        check("idiv.q negative divisor quotient", lo, 0x2aaaaaaaaaaaaaaa);
        check("idiv.q negative divisor remainder", hi, 0xffffffffffffffff);
    }
    {
        hi: u32 = 5;
        lo: u32 = 0x1234;
        d: u32 = 0x10000;
        #asm { div hi, lo, d; }
        check("div.d quotient", lo, 0x50000);
        check("div.d remainder", hi, 0x1234);
    }
    {
        hi: u16 = 0xffff;
        lo: u16 = 0xfc18;
        d: u16 = 7;
        #asm { idiv.w hi, lo, d; }
        check("idiv.w quotient", lo, 0xff72);
        check("idiv.w remainder", hi, 0xfffa);
    }
    {
        ax: u64 = 0xaaaa_0000_0000_03e8; // ax = 1000
        d: u8 = 7;
        #asm { div.b ax, d; }
        check("div.b", ax, 0xaaaa00000000068e);
        bx: u64 = 0xfe0c; // -500
        e: u8 = 7;
        #asm { idiv.b bx, e; }
        check("idiv.b", bx, 0xfdb9);
    }

    // Accumulator widening.
    {
        a: u64 = 0x1111_2222_3333_4480;
        b: u64 = 0x1111_2222_3333_8000;
        c: u64 = 0x1111_2222_8000_0000;
        #asm { cbw a; cwde b; cdqe c; }
        check("cbw", a, 0x111122223333ff80);
        check("cwde", b, 0xffff8000);
        check("cdqe", c, 0xffffffff80000000);
        d16: u64 = 0x5555_5555_5555_5555;
        a16: u64 = 0x8000;
        d32: u64 = 0x5555_5555_5555_5555;
        a32: u64 = 0x7fff_ffff;
        d64: u64 = 0;
        a64: u64 = 0x8000_0000_0000_0000;
        #asm { cwd d16, a16; cdq d32, a32; cqo d64, a64; }
        check("cwd", d16, 0x555555555555ffff);
        check("cdq", d32, 0x0);
        check("cqo", d64, 0xffffffffffffffff);
    }

    // shld / shrd (immediate and register counts) with CF.
    {
        x: u64 = 0x0123_4567_89ab_cdef;
        y: u64 = 0xfedc_ba98_7654_3210;
        c: u8 = 12;
        cf: u8;
        #asm { shld x, y, 20; setc cf; }
        check("shld.q imm", x, 0x56789abcdeffedcb);
        check("shld.q cf", cf, 0x0);
        #asm { shrd y, x, c; setc cf; }
        check("shrd.q reg", y, 0xdcbfedcba9876543);
        check("shrd.q cf", cf, 0x0);
        w: u16 = 0x8421;
        v: u16 = 0xf00f;
        #asm { shld.w w, v, 5; }
        check("shld.w", w, 0x843e);
    }

    // rcl / rcr rotate through the carry flag.
    {
        x: u32 = 0x8000_0001;
        cf: u8;
        #asm { stc; rcl x, 1; setc cf; }
        check("rcl.d 1", x, 0x3);
        check("rcl.d cf", cf, 0x1);
        y: u64 = 0x0123_4567_89ab_cdef;
        #asm { stc; rcr y, 9; setc cf; }
        check("rcr.q 9", y, 0xef8091a2b3c4d5e6);
        check("rcr.q cf", cf, 0x1);
        z: u8 = 0x81;
        #asm { clc; rcl z, 3; setc cf; }
        check("rcl.b 3", z, 0xa);
        check("rcl.b cf", cf, 0x0);
    }

    // BMI1 / BMI2.
    {
        a: u64 = 0xdead_beef_cafe_babe;
        b: u64 = 0x0f0f_0f0f_ff00_ff00;
        r: u64;
        #asm { andn r, b, a; }
        check("andn", r, 0xd0a0b0e000fe00be);
        ctl: u64 = 0x0c08; // start 8, length 12
        #asm { bextr r, a, ctl; }
        check("bextr", r, 0xeba);
        n: u64 = 20;
        #asm { bzhi r, a, n; }
        check("bzhi", r, 0xebabe);
        m: u64 = 0x00ff_00f0_0f0f_f0f0;
        #asm { pdep r, a, m; }
        check("pdep", r, 0xaf00e00b0ab0e0);
        #asm { pext r, a, m; }
        check("pext", r, 0xadeaebb);
        cnt: u64 = 68;
        #asm { shlx r, a, cnt; }
        check("shlx", r, 0xeadbeefcafebabe0);
        #asm { sarx r, a, cnt; }
        check("sarx", r, 0xfdeadbeefcafebab);
        #asm { shrx r, a, cnt; }
        check("shrx", r, 0xdeadbeefcafebab);
        #asm { rorx r, a, 24; }
        check("rorx", r, 0xfebabedeadbeefca);
        hi: u64;
        lo: u64;
        d: u64 = 0x1234_5678_9abc_def0;
        #asm { mulx hi, lo, a, d; }
        check("mulx high", hi, 0xfd5bdeeeb2a01d7);
        check("mulx low", lo, 0xeb689f4ea447d620);
        e: u32 = 0xdead_beef;
        r32: u32;
        f: u32 = 0x00ff_ff00;
        #asm { pext r32, e, f; }
        check("pext.d", r32, 0xadbe);
    }

    // CRC-32C over each source size.
    {
        crc: u32 = 0xffff_ffff;
        b: u8 = 0x61;
        w: u16 = 0x6362;
        d: u32 = 0x6766_6564;
        q: u64 = 0x6f6e_6d6c_6b6a_6968;
        #asm {
            crc32d.b crc, b;
            crc32d.w crc, w;
            crc32d.d crc, d;
        }
        check("crc32 b/w/d", crc, 0x19d80bbe);
        c64: u64 = 0xffff_ffff;
        #asm { crc32q.q c64, q; }
        check("crc32 q", c64, 0x496151a1);
    }

    // Parity, lahf/sahf.
    {
        x: u64 = 0x0102;
        p: u8;
        np: u8;
        #asm { add x, 1; setp p; setnp np; }
        check("setp", p, 0x1);
        check("setnp", np, 0x0);
        y: u64 = 0x10;
        r: u64 = 7;
        #asm { add y, 0x13; cmovp r, y; }
        check("cmovp", r, 0x7);
        a: u64 = 0xffff;
        z: u64 = 5;
        #asm { cmp z, 5; lahf a; }
        check("lahf", a & 0xc7ff, 0x46ff);
        b: u64 = 0x4100; // ZF and CF
        e: u8;
        c: u8;
        #asm { sahf b; sete e; setc c; }
        check("sahf zf", e, 0x1);
        check("sahf cf", c, 0x1);
    }

    // xlat: al = [table + al].
    {
        table: [256] u8;
        for 0..255 table[it] = cast(u8) (255 - it);
        t := table.data;
        al: u64 = 0xabcd_0010;
        #asm { xlat t, al; }
        check("xlat", al, 0xabcd00ef);
    }

    // String instructions.
    {
        src: [64] u8;
        dst: [64] u8;
        for 0..63 src[it] = cast(u8) (it * 3 + 1);
        di := dst.data;
        si := src.data;
        c: u64 = 5;
        #asm {
            cld;
            rep_movs.q di, si, c;
        }
        check("rep_movs.q count", c, 0);
        check("rep_movs.q di", cast(u64) (di - dst.data), 40);
        check("rep_movs.q data", (cast(*u64) (dst.data + 32)).*, 0x7673706d6a676461);

        // Overlapping backward copy (std) moves bytes 0..9 up by 3.
        buf: [16] u8;
        for 0..15 buf[it] = cast(u8) it;
        bd := buf.data + 12;
        bs := buf.data + 9;
        n: u64 = 10;
        #asm {
            std;
            rep_movs.b bd, bs, n;
            cld;
        }
        check("rep_movs.b backward", (cast(*u64) (buf.data + 3)).*, 0x706050403020100);
        check("rep_movs.b backward di", cast(u64) (buf.data + 12 - bd), 10);

        fill: [8] u32;
        fd := fill.data;
        val: u64 = 0x1234_5678_9abc_def0;
        k: u64 = 7;
        #asm { rep_stos.d fd, val, k; }
        check("rep_stos.d", fill[6], 0x9abcdef0);
        check("rep_stos.d untouched", fill[7], 0);

        acc: u64 = 0xffff_ffff_ffff_ffff;
        ls := src.data + 4;
        #asm { lods.w acc, ls; }
        check("lods.w", acc, 0xffffffffffff100d);
        check("lods.w si", cast(u64) (ls - src.data), 6);

        a: [12] u8;
        b: [12] u8;
        for 0..11 {
            a[it] = cast(u8) it;
            b[it] = cast(u8) it;
        }
        b[7] = 99;
        pa := a.data;
        pb := b.data;
        m: u64 = 12;
        below: u8;
        #asm {
            repe_cmps.b pb, pa, m;
            setb below;
        }
        check("repe_cmps.b remaining", m, 0x4);
        check("repe_cmps.b position", cast(u64) (pa - a.data), 0x8);
        check("repe_cmps.b below", below, 0x1);

        text := "hello, world\0";
        p := text.data;
        z: u64 = 0;
        limit: u64 = 100;
        #asm { repne_scas.b p, z, limit; }
        check("repne_scas.b length", 100 - limit - 1, 12);
    }

    // cmpxchg16b: d:a against [mem]; on a match c:b is stored.
    {
        cell: [2] u64 = .[0x1111, 0x2222];
        pcell := cell.data;
        d: u64 = 0x2222;
        a: u64 = 0x1111;
        c: u64 = 0x4444;
        b: u64 = 0x3333;
        ok: u8;
        #asm { lock_cmpxchg16b d, a, [pcell], c, b; setz ok; }
        check("cmpxchg16b success", ok, 1);
        check("cmpxchg16b stored", cell[0] ^ (cell[1] << 16), 0x3333 ^ (0x4444 << 16));
        #asm { cmpxchg16b d, a, [pcell], c, b; setz ok; }
        check("cmpxchg16b failure", ok, 0);
        check("cmpxchg16b loaded", a ^ (d << 16), 0x3333 ^ (0x4444 << 16));
    }

    // adcx / adox carry through CF and OF independently (computed by hand: no ADX under Rosetta).
    {
        x: u64 = 0xffff_ffff_ffff_ffff;
        y: u64 = 0x8000_0000_0000_0000;
        one: u64 = 1;
        cf: u8;
        of: u8;
        #asm {
            stc;
            xor z:, z;        // clears CF and OF
            stc;              // CF = 1, OF = 0
            adcx x, one;      // x = 0xffff... + 1 + 1 = 1, CF = 1
            adox y, y;        // y = 0, OF = 1
            setc cf;
            seto of;
        }
        check("adcx", x, 1);
        check("adox", y, 0);
        check("adcx cf", cf, 1);
        check("adox of", of, 1);
    }

    if failures exit(1);
    print("ok\n");
}
