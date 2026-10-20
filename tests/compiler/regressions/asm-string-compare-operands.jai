// SSE4.2 string compares with memory operands, and the index, mask and flag results of the
// common search modes, each checked against a value worked out from the instruction's definition.
// rules: asm.30
#import "Basic";

failures := 0;

expect :: (name: string, got: s64, want: s64) {
    if got == want return;
    failures += 1;
    print("FAIL %: got %, want %\n", name, got, want);
}

main :: () {
    idx: u32;
    cf, zf, sf, of: u8;
    mask: [16] u8;
    pm := mask.data;

    // Equal ordered, bytes: the first place "lo" occurs in "hello world". Both strings end
    // before the sixteenth byte, so ZF and SF are set; the match does not start at 0, so OF is 0.
    {
        needle := u8.[108, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        hay := u8.[104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0];
        pa := needle.data;
        pb := hay.data;

        #asm SSE4_2 {
            pcmpistri ecx:, [pa], [pb], 12;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
            seto of;
        }

        expect("ordered index", idx, 3);
        expect("ordered cf", cf, 1);
        expect("ordered zf", zf, 1);
        expect("ordered sf", sf, 1);
        expect("ordered of", of, 0);
    }

    // Ranges, bytes: the first and the last character of "123abc" within 'a'..'z'.
    {
        range := u8.[97, 122, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        text := u8.[49, 50, 51, 97, 98, 99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pa := range.data;
        pb := text.data;

        #asm SSE4_2 {
            a_: vec;
            movdqu a_, [pa];
            pcmpistri ecx:, a_, [pb], 4;
            mov idx, ecx;
        }

        expect("range lowest", idx, 3);

        #asm SSE4_2 {
            a_: vec;
            movdqu a_, [pa];
            pcmpistri ecx:, a_, [pb], 68;
            mov idx, ecx;
        }

        expect("range highest", idx, 5);
    }

    // Signed against unsigned ranges: -5..5 holds -2, while 251..5 holds nothing unsigned.
    {
        range := u8.[251, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        text := u8.[128, 254, 1, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pa := range.data;
        pb := text.data;

        #asm SSE4_2 {
            pcmpistri ecx:, [pa], [pb], 6;
            mov idx, ecx;
            setc cf;
        }

        expect("signed range index", idx, 1);
        expect("signed range cf", cf, 1);

        #asm SSE4_2 {
            pcmpistri ecx:, [pa], [pb], 4;
            mov idx, ecx;
            setc cf;
        }

        expect("unsigned range index", idx, 16);
        expect("unsigned range cf", cf, 0);
    }

    // Polarity: with "abc" on both sides, negating every element sets the invalid ones too,
    // while the masked negation leaves them alone.
    {
        text := u8.[97, 98, 99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pa := text.data;
        pb := text.data;

        #asm SSE4_2 {
            pcmpistri ecx:, [pa], [pb], 16;
            mov idx, ecx;
            setc cf;
        }

        expect("negated index", idx, 3);
        expect("negated cf", cf, 1);

        #asm SSE4_2 {
            pcmpistri ecx:, [pa], [pb], 48;
            mov idx, ecx;
            setc cf;
        }

        expect("masked negated index", idx, 16);
        expect("masked negated cf", cf, 0);
    }

    // Equal any as masks: the vowels of "education", as one bit per byte and as whole bytes.
    {
        vowels := u8.[97, 101, 105, 111, 117, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        word := u8.[101, 100, 117, 99, 97, 116, 105, 111, 110, 0, 0, 0, 0, 0, 0, 0];
        pa := vowels.data;
        pb := word.data;

        #asm SSE4_2 {
            m0: vec;
            pcmpistrm m0, [pa], [pb], 0;
            movdqu [pm], m0;
        }

        expect("bit mask low", mask[0], 0xd5);
        expect("bit mask high", mask[1], 0);

        #asm SSE4_2 {
            m0: vec;
            pcmpistrm m0, [pa], [pb], 64;
            movdqu [pm], m0;
        }

        expect("byte mask 0", mask[0], 255);
        expect("byte mask 1", mask[1], 0);
        expect("byte mask 2", mask[2], 255);
        expect("byte mask 3", mask[3], 0);
        expect("byte mask 4", mask[4], 255);
        expect("byte mask 7", mask[7], 255);
        expect("byte mask 8", mask[8], 0);
    }

    // Equal each with explicit lengths: "abc.." against "abd.." over three elements, then the
    // same with the result negated, which leaves only the mismatching third element.
    {
        left := u8.[97, 98, 99, 46, 46, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        right := u8.[97, 98, 100, 46, 46, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        pa := left.data;
        pb := right.data;
        la: s32 = 3;
        lb: s32 = 3;

        #asm SSE4_2 {
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, [pa], [pb], 24;
            mov idx, ecx;
            setc cf;
            setz zf;
            sets sf;
        }

        expect("each negated index", idx, 2);
        expect("each negated cf", cf, 1);
        expect("each zf", zf, 1);
        expect("each sf", sf, 1);

        #asm SSE4_2 {
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, [pa], [pb], 8;
            mov idx, ecx;
        }

        expect("each index", idx, 0);

        // Full-length strings clear ZF and SF, and a negative length counts as its magnitude.
        la = -16;
        lb = 16;

        #asm SSE4_2 {
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            pcmpestri ecx:, ea, ed, [pa], [pb], 8;
            setz zf;
            sets sf;
        }

        expect("full zf", zf, 0);
        expect("full sf", sf, 0);
    }

    // Words: equal any over 16-bit elements, and the explicit-length mask form.
    {
        wanted := u16.[1, 2, 0, 0, 0, 0, 0, 0];
        haystack := u16.[5, 2, 1, 9, 0, 0, 0, 0];
        pa := wanted.data;
        pb := haystack.data;

        #asm SSE4_2 {
            pcmpistri ecx:, [pa], [pb], 1;
            mov idx, ecx;
        }

        expect("words index", idx, 1);

        la: s32 = 2;
        lb: s32 = 4;

        #asm SSE4_2 {
            ea: gpr;
            ed: gpr;
            mov ea, la;
            mov ed, lb;
            m0: vec;
            pcmpestrm m0, ea, ed, [pa], [pb], 65;
            movdqu [pm], m0;
        }

        expect("word mask 0", mask[0], 0);
        expect("word mask 2", mask[2], 255);
        expect("word mask 3", mask[3], 255);
        expect("word mask 4", mask[4], 255);
        expect("word mask 6", mask[6], 0);
    }

    if failures != 0 exit(1);

    print("ok\n");
}
