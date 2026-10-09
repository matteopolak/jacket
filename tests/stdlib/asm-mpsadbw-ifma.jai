// mpsadbw (SSE4.1 128-bit and AVX2 256-bit, with every immediate) and the AVX-512 IFMA
// pmadd52luq / pmadd52huq. The 128-bit mpsadbw results come from x86-64 hardware (the
// _mm_mpsadbw_epu8 intrinsic); the 256-bit form and IFMA, which that machine lacks, from the
// documented definitions.
// rules: asm.31
#import "Basic";

failures := 0;

same :: (name: string, got: [] u16, want: [] u16) {
    for 0..want.count - 1 {
        if got[it] != want[it] {
            print("FAIL %: word % is %, want %\n", name, it, got[it], want[it]);
            failures += 1;
            return;
        }
    }
}

same64 :: (name: string, got: [] u64, want: [] u64) {
    for 0..want.count - 1 {
        if got[it] != want[it] {
            print("FAIL %: lane % is 0x%, want 0x%\n", name, it, formatInt(got[it], base = 16), formatInt(want[it], base = 16));
            failures += 1;
            return;
        }
    }
}

main :: () {
    A: [32] u8;
    B: [32] u8;
    for 0..31 {
        A[it] = xx ((it * 37 + 11) & 255);
        B[it] = xx ((it * 91 + 5) & 255);
    }
    out: [16] u16;
    pa, pb, po := A.data, B.data, cast(*u16) out.data;

    #asm SSE4_1 {
        movdqu.x a:, [pa];
        movdqu.x b:, [pb];
        mpsadbw.x a, b, 0;
        movdqu.x [po], a;
    }
    same("mpsadbw 0", .[out[0], out[1], out[2], out[3], out[4], out[5], out[6], out[7]], .[256, 256, 308, 400, 308, 530, 512, 256]);
    #asm SSE4_1 {
        movdqu.x a:, [pa];
        movdqu.x b:, [pb];
        mpsadbw.x a, b, 1;
        movdqu.x [po], a;
    }
    same("mpsadbw 1", .[out[0], out[1], out[2], out[3], out[4], out[5], out[6], out[7]], .[312, 296, 296, 314, 364, 216, 364, 306]);
    #asm SSE4_1 {
        movdqu.x a:, [pa];
        movdqu.x b:, [pb];
        mpsadbw.x a, b, 2;
        movdqu.x [po], a;
    }
    same("mpsadbw 2", .[out[0], out[1], out[2], out[3], out[4], out[5], out[6], out[7]], .[396, 306, 256, 256, 512, 522, 300, 384]);
    #asm SSE4_1 {
        movdqu.x a:, [pa];
        movdqu.x b:, [pb];
        mpsadbw.x a, b, 3;
        movdqu.x [po], a;
    }
    same("mpsadbw 3", .[out[0], out[1], out[2], out[3], out[4], out[5], out[6], out[7]], .[380, 306, 256, 256, 216, 472, 516, 374]);
    #asm SSE4_1 {
        movdqu.x a:, [pa];
        movdqu.x b:, [pb];
        mpsadbw.x a, b, 4;
        movdqu.x [po], a;
    }
    same("mpsadbw 4", .[out[0], out[1], out[2], out[3], out[4], out[5], out[6], out[7]], .[308, 530, 512, 256, 256, 314, 412, 314]);
    #asm SSE4_1 {
        movdqu.x a:, [pa];
        movdqu.x b:, [pb];
        mpsadbw.x a, b, 5;
        movdqu.x [po], a;
    }
    same("mpsadbw 5", .[out[0], out[1], out[2], out[3], out[4], out[5], out[6], out[7]], .[364, 216, 364, 306, 296, 296, 320, 364]);
    #asm SSE4_1 {
        movdqu.x a:, [pa];
        movdqu.x b:, [pb];
        mpsadbw.x a, b, 6;
        movdqu.x [po], a;
    }
    same("mpsadbw 6", .[out[0], out[1], out[2], out[3], out[4], out[5], out[6], out[7]], .[512, 522, 300, 384, 300, 256, 256, 512]);
    #asm SSE4_1 {
        movdqu.x a:, [pa];
        movdqu.x b:, [pb];
        mpsadbw.x a, b, 7;
        movdqu.x [po], a;
    }
    same("mpsadbw 7", .[out[0], out[1], out[2], out[3], out[4], out[5], out[6], out[7]], .[216, 472, 516, 374, 300, 256, 256, 216]);
    #asm AVX2 {
        vmovdqu.y a:, [pa];
        vmovdqu.y b:, [pb];
        vmpsadbw.y c:, a, b, 0;
        vmovdqu.y [po], c;
    }
    same("vmpsadbw.y 0", out, .[256, 256, 308, 400, 308, 530, 512, 256, 264, 338, 512, 472, 216, 282, 256, 270]);
    #asm AVX2 {
        vmovdqu.y a:, [pa];
        vmovdqu.y b:, [pb];
        vmpsadbw.y c:, a, b, 9;
        vmovdqu.y [po], c;
    }
    same("vmpsadbw.y 9", out, .[312, 296, 296, 314, 364, 216, 364, 306, 264, 338, 264, 486, 512, 282, 256, 270]);
    #asm AVX2 {
        vmovdqu.y a:, [pa];
        vmovdqu.y b:, [pb];
        vmpsadbw.y c:, a, b, 27;
        vmovdqu.y [po], c;
    }
    same("vmpsadbw.y 27", out, .[380, 306, 256, 256, 216, 472, 516, 374, 404, 458, 276, 404, 276, 458, 404, 404]);
    #asm AVX2 {
        vmovdqu.y a:, [pa];
        vmovdqu.y b:, [pb];
        vmpsadbw.y c:, a, b, 36;
        vmovdqu.y [po], c;
    }
    same("vmpsadbw.y 36", out, .[308, 530, 512, 256, 256, 314, 412, 314, 216, 282, 256, 270, 344, 512, 472, 216]);
    #asm AVX2 {
        vmovdqu.y a:, [pa];
        vmovdqu.y b:, [pb];
        vmpsadbw.y c:, a, b, 45;
        vmovdqu.y [po], c;
    }
    same("vmpsadbw.y 45", out, .[364, 216, 364, 306, 296, 296, 320, 364, 512, 282, 256, 270, 344, 270, 492, 512]);
    #asm AVX2 {
        vmovdqu.y a:, [pa];
        vmovdqu.y b:, [pb];
        vmpsadbw.y c:, a, b, 63;
        vmovdqu.y [po], c;
    }
    same("vmpsadbw.y 63", out, .[216, 472, 516, 374, 300, 256, 256, 216, 276, 458, 404, 404, 464, 282, 404, 270]);

    X: [8] u64 = .[0x0, 0x1, 0xfffffffffffff, 0xfedcba9876543, 0xfff0000000000001, 0x8000000000000, 0x1234567890abc, 0xdeadbeefcafef00d];
    Y: [8] u64 = .[0xfffffffffffff, 0x3, 0xfffffffffffff, 0x123456789abcd, 0x7, 0x8000000000001, 0xfedcba987654, 0xbadf00dd15ea5e5];
    R: [8] u64;
    px, py, pr := X.data, Y.data, R.data;
    seed00: u64 = 0x0;
    #asm AVX512F {
        vmovdqu64.z x:, [px];
        vmovdqu64.z y:, [py];
        acc: vec;
        vpbroadcastq.z acc, seed00;
        vpmadd52luq.z acc, x, y;
        vmovdqu64.z [pr], acc;
    }
    same64("vpmadd52luq acc 0x0", R, .[0x0, 0x3, 0x1, 0x51d9657ead7a7, 0x7, 0x8000000000000, 0xc5d3a1b8a2db0, 0xa6696a88b1ca1]);
    seed01: u64 = 0x0;
    #asm AVX512F {
        vmovdqu64.z x:, [px];
        vmovdqu64.z y:, [py];
        acc: vec;
        vpbroadcastq.z acc, seed01;
        vpmadd52huq.z acc, x, y;
        vmovdqu64.z [pr], acc;
    }
    same64("vpmadd52huq acc 0x0", R, .[0x0, 0x0, 0xffffffffffffe, 0x121fa00ad77d6, 0x0, 0x4000000000000, 0x121fa00acd77, 0xbf95ec0d6549c]);
    seed10: u64 = 0x5;
    #asm AVX512F {
        vmovdqu64.z x:, [px];
        vmovdqu64.z y:, [py];
        acc: vec;
        vpbroadcastq.z acc, seed10;
        vpmadd52luq.z acc, x, y;
        vmovdqu64.z [pr], acc;
    }
    same64("vpmadd52luq acc 0x5", R, .[0x5, 0x8, 0x6, 0x51d9657ead7ac, 0xc, 0x8000000000005, 0xc5d3a1b8a2db5, 0xa6696a88b1ca6]);
    seed11: u64 = 0x5;
    #asm AVX512F {
        vmovdqu64.z x:, [px];
        vmovdqu64.z y:, [py];
        acc: vec;
        vpbroadcastq.z acc, seed11;
        vpmadd52huq.z acc, x, y;
        vmovdqu64.z [pr], acc;
    }
    same64("vpmadd52huq acc 0x5", R, .[0x5, 0x5, 0x10000000000003, 0x121fa00ad77db, 0x5, 0x4000000000005, 0x121fa00acd7c, 0xbf95ec0d654a1]);

    if failures != 0 exit(1);
    print("ok\n");
}
