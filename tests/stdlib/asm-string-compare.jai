// SSE4.2 string compares (pcmpistri, pcmpestri, pcmpistrm, pcmpestrm) over bytes and words, with
// every aggregation mode and polarity, implicit and explicit lengths, and the flags they set.
// Each case's outputs are written as text and hashed (FNV-1a); the expected hash comes from the
// same calls (the _mm_cmp[ie]str intrinsics) on x86-64 hardware.
// rules: asm.30
#import "Basic";

DATA :: u8.[
    98, 98, 99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    97, 97, 97, 97, 98, 97, 99, 97, 98, 97, 99, 98, 97, 98, 98, 97,
    122, 57, 48, 57, 90, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    97, 77, 97, 97, 53, 97, 97, 45, 45, 77, 53, 56, 53, 97, 0, 0,
    104, 101, 108, 108, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    115, 97, 121, 32, 104, 101, 108, 108, 111, 32, 104, 101, 108, 108, 111, 0,
    98, 98, 97, 97, 98, 97, 98, 98, 97, 98, 98, 98, 98, 97, 98, 98,
    98, 97, 98, 98, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    128, 127, 1, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    144, 127, 0, 254, 129, 126, 255, 5, 1, 2, 3, 4, 5, 6, 7, 8,
    1, 0, 2, 0, 3, 0, 0, 0, 9, 9, 9, 9, 9, 9, 9, 9,
    2, 0, 1, 0, 3, 0, 4, 0, 1, 0, 2, 0, 3, 0, 0, 0,
    0, 128, 255, 127, 1, 0, 254, 255, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 144, 255, 127, 1, 0, 16, 0, 5, 0, 255, 255, 0, 128, 1, 0,
];

HEX :: "0123456789abcdef";

emit :: (out: *String_Builder, pi: int, ctl: int, form: int, idx: u32, cf: u8, zf: u8, sf: u8, of: u8, mask: [16] u8) {
    print_to_builder(out, "% % % % %%%% ", pi, ctl, form, idx, cf, zf, sf, of);
    for mask print_to_builder(out, "% ", it);
    print_to_builder(out, "\n");
}

main :: () {
    data := DATA;
    sb: String_Builder;
    mask: [16] u8;
    idx: u32;
    cf, zf, sf, of: u8;

    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 0;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 0, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 0;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 48;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 48, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 36;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 36, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 0;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 24;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 24, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 108;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 108, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 3;
        lb: s32 = 7;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 1;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 1, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 5;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 5, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 117;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 117, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 57;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 57, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 93;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 93, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 66;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 66, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 0;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 6;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 6, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 10;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 10, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 0;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 42;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 42, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 30;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 30, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 99;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 99, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 87;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 87, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 11;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 11, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 123;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 123, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 0;
        pb := data.data + 16;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 63;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 0, 63, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 48;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 48, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 3;
        lb: s32 = 7;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 84;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 84, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 72;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 72, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 3;
        lb: s32 = 7;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 12;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 12, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 1;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 1, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 8;
        lb: s32 = 2;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 33;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 33, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 21;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 21, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 8;
        lb: s32 = 2;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 105;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 105, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 93;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 93, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 2;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 2, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 114;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 114, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 3;
        lb: s32 = 7;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 54;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 54, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 42;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 42, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 3;
        lb: s32 = 7;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 78;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 78, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 3;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 3, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 7;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 7, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 11;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 11, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 8;
        lb: s32 = 2;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 27;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 27, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 32;
        pb := data.data + 48;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 111;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 1, 111, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 96;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 96, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 84;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 84, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 8;
        lb: s32 = 2;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 8;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 8, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 120;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 120, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 60;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 60, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 33;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 33, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 69;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 69, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 9;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 9, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 13;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 13, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 2;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 2, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 18;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 18, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 102;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 102, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 90;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 90, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 78;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 78, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 126;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 126, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 51;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 51, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 39;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 39, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 27;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 27, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 64;
        pb := data.data + 80;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 15;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 2, 15, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 0;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 0, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 3;
        lb: s32 = 7;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 4;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 4, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 8;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 8, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 24;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 24, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 108;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 108, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 0;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 81;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 81, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 69;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 69, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 0;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 117;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 117, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 57;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 57, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 0;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 45;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 45, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 18;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 18, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 6;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 6, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 10;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 10, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 14;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 14, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 126;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 126, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 0;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 99;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 99, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 87;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 87, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 0;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 75;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 75, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 15;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 15, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 96;
        pb := data.data + 112;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 0;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 63;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 3, 63, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 48;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 48, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 8;
        lb: s32 = 2;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 36;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 36, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 24;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 24, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 8;
        lb: s32 = 2;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 12;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 12, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 1;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 1, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 5;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 5, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 117;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 117, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 3;
        lb: s32 = 7;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 105;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 105, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 93;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 93, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 8;
        lb: s32 = 2;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 66;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 66, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 6;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 6, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 8;
        lb: s32 = 2;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 54;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 54, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 42;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 42, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 8;
        lb: s32 = 2;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 30;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 30, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 99;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 99, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 7;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 7, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 11;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 11, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 3;
        lb: s32 = 7;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 123;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 123, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 128;
        pb := data.data + 144;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 63;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 4, 63, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 96;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 96, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 84;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 84, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 72;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 72, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 12;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 12, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 60;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 60, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 33;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 33, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 21;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 21, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 105;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 105, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 0;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 13;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 13, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 2;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 2, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 114;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 114, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 54;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 54, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 90;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 90, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 78;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 78, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 3;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 3, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 7;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 7, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 39;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 39, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 27;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 27, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 160;
        pb := data.data + 176;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 111;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 5, 111, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 96;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 96, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 8;
        lb: s32 = 2;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 4;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 4, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 8;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 8, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 0;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 120;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 120, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 60;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 60, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 81;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 81, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 69;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 69, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 9;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 9, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 13;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 13, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 45;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 45, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 18;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 18, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 0;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 102;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 102, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 90;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 90, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 14;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 14, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 126;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 126, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 51;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 51, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 39;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 39, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 75;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 75, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 192;
        pb := data.data + 208;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 15;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 6, 15, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 3;
        lb: s32 = 7;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 0;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 0, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 4;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 4, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 3;
        lb: s32 = 7;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 36;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 36, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 24;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 24, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 3;
        lb: s32 = 7;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 108;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 108, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 81;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 81, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 0;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 5;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 5, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 117;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 117, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 8;
        lb: s32 = 2;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 57;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 57, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 45;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 45, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 3;
        lb: s32 = 7;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 66;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 66, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 6;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 6, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 10;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 10, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 14;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 14, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 3;
        lb: s32 = 7;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 30;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 30, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 99;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 99, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 8;
        lb: s32 = 2;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 87;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 87, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 75;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 75, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 8;
        lb: s32 = 2;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 123;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 123, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 224;
        pb := data.data + 240;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 63;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 7, 63, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 48;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 48, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 36;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 36, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 72;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 72, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 12;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 12, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = -5;
        lb: s32 = 20;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 1;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 1, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 5;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 5, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 21;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 21, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 105;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 105, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 93;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 93, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 66;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 66, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 114;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 114, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 54;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 54, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 16;
        lb: s32 = 16;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 42;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 42, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 30;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 30, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 3;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 3, 3, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            m0: vec;
            pcmpistrm m0, a_, b_, 7;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 7, 2, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        la: s32 = 3;
        lb: s32 = 7;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, a_, b_, 11;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 11, 1, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            pcmpistri ecx:, a_, b_, 123;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 123, 0, idx, cf, zf, sf, of, mask);
    }
    {
        pa := data.data + 256;
        pb := data.data + 272;
        idx = 0;
        cf = 0;
        zf = 0;
        sf = 0;
        of = 0;
        mask = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pm := mask.data;
        la: s32 = 1;
        lb: s32 = 0;
        #asm SSE4_2 {
            a_: vec;
            b_: vec;
            movdqu a_, [pa];
            movdqu b_, [pb];
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, a_, b_, 111;
            movdqu [pm], m0;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }
        emit(*sb, 8, 111, 3, idx, cf, zf, sf, of, mask);
    }

    text := builder_to_string(*sb);
    h: u64 = 0xcbf2_9ce4_8422_2325;
    for 0..text.count - 1 {
        h ^= text[it];
        h *= 0x100_0000_01b3;
    }
    if text.count != 8733 || h != 0xc53cdae5dcfc2c91 {
        print("FAIL: % bytes, hash 0x%\n", text.count, formatInt(h, base = 16));
        exit(1);
    }
    print("ok\n");
}
