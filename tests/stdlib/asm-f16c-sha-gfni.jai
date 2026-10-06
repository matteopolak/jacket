// rules: asm.26
// F16C, SHA and GFNI #asm instructions against answers that do not come from the
// implementation: FIPS 180-4 digests computed with the SHA message and round
// instructions, GF(2^8) products and affine maps against plain Jai arithmetic and
// the AES S-box, and half-precision conversions against exact values and a
// rounding reference built from float64 comparisons. The F16C hashes were recorded
// with the same sequences on x86-64 (Rosetta 2 with ROSETTA_ADVERTISE_AVX=1, which
// implements F16C); SHA and GFNI had no x86 hardware available.
#import "Basic";

failures := 0;

fnv :: (h: *u64, p: *u8, n: int) {
    for 0..n - 1 {
        h.* ^= p[it];
        h.* *= 0x100_0000_01b3;
    }
}

expect :: (name: string, got: u64, want: u64) {
    if got != want {
        print("FAIL %: got 0x%, want 0x%\n", name, formatInt(got, base = 16), formatInt(want, base = 16));
        failures += 1;
    }
}

// ---- F16C ----

widen8 :: (src: *u16, dst: *u32) {
    #asm F16C, AVX {
        vcvtph2ps.y v:, [src];
        movdqu.y [dst], v;
    }
}

narrow8 :: (src: *u32, dst: *u16, $mode: int) {
    #asm F16C, AVX {
        movdqu.y v:, [src];
        vcvtps2ph.y [dst], v, mode;
    }
}

to_half :: (bits: u32, $mode: int) -> u16 {
    src: [8] u32;
    dst: [8] u16;
    src[0] = bits;
    narrow8(src.data, dst.data, mode);
    return dst[0];
}

to_single :: (h: u16) -> u32 {
    src: [8] u16;
    dst: [8] u32;
    src[0] = h;
    widen8(src.data, dst.data);
    return dst[0];
}

bits_of :: (x: float32) -> u32 {
    v := x;
    return (cast(*u32) *v).*;
}

float_of :: (b: u32) -> float32 {
    v := b;
    return (cast(*float32) *v).*;
}

times_pow2 :: (x: float64, n: int) -> float64 {
    r := x;
    if n > 0 for 1..n r *= 2;
    if n < 0 for 1..-n r *= 0.5;
    return r;
}

// The value of a finite half with the sign dropped; 0x7c00 stands for 2^16, the step
// past the largest finite half that decides overflow rounding.
half_magnitude :: (h: u16) -> float64 {
    e := cast(int) ((h >> 10) & 0x1f);
    m := cast(int) (h & 0x3ff);
    if e == 0 return times_pow2(cast(float64) m, -24);
    return times_pow2(cast(float64) (1024 + m), e - 25);
}

// Reference rounding for finite and infinite inputs: find the halves around |x| by
// binary search and pick one according to the mode (0 nearest even, 1 down, 2 up,
// 3 toward zero).
reference_half :: (bits: u32, mode: int) -> u16 {
    negative := (bits >> 31) != 0;
    sign: u16 = ifx negative then cast(u16) 0x8000 else 0;
    if (bits & 0x7fff_ffff) == 0x7f80_0000 return sign | 0x7c00;
    a := cast(float64) float_of(bits & 0x7fff_ffff);
    lo: u16 = 0;
    hi: u16 = 0x7bff;
    if half_magnitude(hi) <= a {
        lo = hi;
    } else {
        while lo < hi {
            mid := cast(u16) ((lo + hi + 1) / 2);
            if half_magnitude(mid) <= a lo = mid;
            else hi = mid - 1;
        }
    }
    below := half_magnitude(lo);
    if below == a return sign | lo;
    above := half_magnitude(lo + 1);
    up := false;
    if mode == {
        case 0;
            middle := (below + above) / 2;
            up = a > middle || (a == middle && (lo & 1) != 0);
        case 1; up = negative;
        case 2; up = !negative;
    }
    if up lo += 1;
    return sign | lo;
}

check_f16c :: () {
    // Exact values in both directions.
    pairs :: u32.[
        0x3f80_0000, 0x3c00, // 1
        0xc000_0000, 0xc000, // -2
        0x3f00_0000, 0x3800, // 0.5
        0x477f_e000, 0x7bff, // 65504
        0x3880_0000, 0x0400, // 2^-14, smallest normal
        0x3380_0000, 0x0001, // 2^-24, smallest subnormal
        0x387f_c000, 0x03ff, // largest subnormal
        0x3800_0000, 0x0200, // 2^-15
        0x8000_0000, 0x8000, // -0
        0x7f80_0000, 0x7c00, // inf
        0xff80_0000, 0xfc00, // -inf
        0x7fc0_0000, 0x7e00, // quiet NaN
    ];
    for i: 0..pairs.count / 2 - 1 {
        s := pairs[2 * i];
        h := cast(u16) pairs[2 * i + 1];
        expect(tprint("cvtph2ps %", formatInt(h, base = 16)), to_single(h), s);
        expect(tprint("cvtps2ph %", formatInt(s, base = 16)), to_half(s, 0), h);
    }

    // NaNs come back quiet with their payload truncated or widened.
    expect("signaling half NaN", to_single(0x7d00), 0x7fe0_0000);
    expect("negative half NaN", to_single(0xfc01), 0xffc0_2000);
    expect("signaling single NaN", to_half(0x7f81_2345, 0), 0x7e09);
    expect("negative single NaN", to_half(0xffc0_0001, 3), 0xfe00);

    // Rounding modes on ties, overflow and underflow.
    one_and_half_ulp :: 0x3f80_1000; // 1 + 2^-11: halfway between 0x3c00 and 0x3c01
    expect("tie even", to_half(one_and_half_ulp, 0), 0x3c00);
    expect("tie down", to_half(one_and_half_ulp, 1), 0x3c00);
    expect("tie up", to_half(one_and_half_ulp, 2), 0x3c01);
    expect("tie zero", to_half(one_and_half_ulp, 3), 0x3c00);
    expect("tie odd", to_half(0x3f80_3000, 0), 0x3c02);
    expect("neg tie down", to_half(0xbf80_1000, 1), 0xbc01);
    expect("neg tie up", to_half(0xbf80_1000, 2), 0xbc00);
    expect("mxcsr mode", to_half(one_and_half_ulp, 6), 0x3c00);
    expect("65519 nearest", to_half(bits_of(65519), 0), 0x7bff);
    expect("65520 nearest", to_half(bits_of(65520), 0), 0x7c00);
    expect("65520 zero", to_half(bits_of(65520), 3), 0x7bff);
    expect("big up", to_half(bits_of(1e10), 2), 0x7c00);
    expect("big down", to_half(bits_of(1e10), 1), 0x7bff);
    expect("-big down", to_half(bits_of(-1e10), 1), 0xfc00);
    expect("-big up", to_half(bits_of(-1e10), 2), 0xfbff);
    expect("-big zero", to_half(bits_of(-1e10), 3), 0xfbff);
    expect("2^-25 nearest", to_half(0x3300_0000, 0), 0x0000);
    expect("2^-25 up", to_half(0x3300_0000, 2), 0x0001);
    expect("1.5 * 2^-24", to_half(0x33c0_0000, 0), 0x0002);
    expect("0.75 * 2^-24", to_half(0x3340_0000, 0), 0x0001);
    expect("subnormal to normal", to_half(0x387f_e000, 0), 0x0400);
    expect("tiny single up", to_half(0x0000_0001, 2), 0x0001);
    expect("tiny single nearest", to_half(0x0000_0001, 0), 0x0000);
    expect("-tiny single down", to_half(0x8000_0001, 1), 0x8001);

    // Every half widens to its exact value and narrows back to itself.
    halves: [8] u16;
    singles: [8] u32;
    back: [8] u16;
    mismatches := 0;
    widened: u64 = 0xcbf2_9ce4_8422_2325;
    for base: 0..8191 {
        for j: 0..7 halves[j] = cast(u16) (base * 8 + j);
        widen8(halves.data, singles.data);
        fnv(*widened, cast(*u8) singles.data, 32);
        narrow8(singles.data, back.data, 0);
        for j: 0..7 {
            h := halves[j];
            s := singles[j];
            is_nan := (h & 0x7c00) == 0x7c00 && (h & 0x3ff) != 0;
            want: u32;
            if is_nan {
                want = ((cast(u32) h & 0x8000) << 16) | 0x7fc0_0000 | ((cast(u32) h & 0x3ff) << 13);
            } else if (h & 0x7fff) == 0x7c00 {
                want = ((cast(u32) h & 0x8000) << 16) | 0x7f80_0000;
            } else {
                v := cast(float32) half_magnitude(h & 0x7fff);
                want = bits_of(v) | ((cast(u32) h & 0x8000) << 16);
            }
            want_back := ifx is_nan then h | 0x200 else h;
            if s != want || back[j] != want_back mismatches += 1;
        }
    }
    expect("half round trips", cast(u64) mismatches, 0);
    // FNV-1a of the widened values, recorded with vcvtph2ps on x86-64.
    expect("cvtph2ps hardware hash", widened, 0x5d79_f1b0_86f3_0345);

    // vcvtps2ph in each rounding mode over pseudo-random singles (odd lanes fully
    // random, so NaNs and infinities too); hashes recorded on x86-64.
    recorded :: u64.[0x87ba_3965_eff5_b47f, 0xeb2e_4270_3c09_0389, 0xd7f4_0491_ecaf_f047, 0x82d1_9a17_2adf_cb7c, 0x87ba_3965_eff5_b47f];
    for mode: 0..4 {
        hash: u64 = 0xcbf2_9ce4_8422_2325;
        seed: u32 = 0x2468_ace1;
        step :: () -> u32 #expand {
            `seed ^= `seed << 13;
            `seed ^= `seed >> 17;
            `seed ^= `seed << 5;
            return `seed;
        }
        lanes: [8] u32;
        halves8: [8] u16;
        for 1..2048 {
            for j: 0..7 {
                r := step();
                e := 96 + (step() & 63);
                lanes[j] = ifx j & 1 then r else (r & 0x807f_ffff) | (e << 23);
            }
            if mode == {
                case 0; narrow8(lanes.data, halves8.data, 0);
                case 1; narrow8(lanes.data, halves8.data, 1);
                case 2; narrow8(lanes.data, halves8.data, 2);
                case 3; narrow8(lanes.data, halves8.data, 3);
                case; narrow8(lanes.data, halves8.data, 4);
            }
            fnv(*hash, cast(*u8) halves8.data, 16);
        }
        expect(tprint("cvtps2ph hardware hash %", mode), hash, recorded[mode]);
    }

    // Pseudo-random singles around the half range, all modes, against the reference.
    state: u32 = 0x1234_5678;
    next :: () -> u32 #expand {
        `state ^= `state << 13;
        `state ^= `state >> 17;
        `state ^= `state << 5;
        return `state;
    }
    src: [8] u32;
    out: [5][8] u16;
    wrong := 0;
    for 1..400 {
        for j: 0..7 {
            r := next();
            // Exponents 96..159 cover subnormal halves, normals and overflow.
            e := 96 + (next() & 63);
            src[j] = (r & 0x807f_ffff) | (e << 23);
            if j == 7 src[j] = next();
        }
        narrow8(src.data, out[0].data, 0);
        narrow8(src.data, out[1].data, 1);
        narrow8(src.data, out[2].data, 2);
        narrow8(src.data, out[3].data, 3);
        narrow8(src.data, out[4].data, 4);
        for j: 0..7 {
            x := src[j];
            if (x & 0x7f80_0000) == 0x7f80_0000 && (x & 0x7f_ffff) != 0 continue;
            for mode: 0..3 if out[mode][j] != reference_half(x, mode) wrong += 1;
            if out[4][j] != out[0][j] wrong += 1;
        }
    }
    expect("cvtps2ph against reference", cast(u64) wrong, 0);

    // xmm forms read and write half as much; a register result is zero above it.
    {
        hs: [8] u16 = .[0x3c00, 0x4000, 0x4200, 0x4400, 0x4500, 0x4600, 0x4700, 0x4800];
        fs: [8] float32;
        for *fs it.* = -1;
        ph, pf := hs.data, fs.data;
        #asm F16C {
            vcvtph2ps.x v:, [ph];
            movdqu.x [pf], v;
        }
        expect("cvtph2ps xmm", cast(u64) (fs[0] == 1 && fs[3] == 4 && fs[4] == -1), 1);
        hs2: [8] u16;
        for *hs2 it.* = 0xffff;
        ph2 := hs2.data;
        #asm F16C {
            movdqu.x v, [pf];
            vcvtps2ph.x [ph2], v, 0;
        }
        expect("cvtps2ph m64", cast(u64) (hs2[0] == 0x3c00 && hs2[3] == 0x4400 && hs2[4] == 0xffff), 1);
        wide: [8] u32;
        for *wide it.* = 0xffff_ffff;
        pw := wide.data;
        #asm F16C, AVX {
            movdqu.y r:, [pw];
            movdqu.x v, [pf];
            vcvtps2ph.x r, v, 0;
            movdqu.y [pw], r;
        }
        expect("cvtps2ph xmm clears", cast(u64) (wide[0] == 0x4000_3c00 && wide[2] == 0 && wide[7] == 0), 1);
    }

    // EVEX: sixteen lanes and write masks on half-sized elements.
    {
        fs: [16] float32;
        for 0..15 fs[it] = cast(float32) it;
        hs: [16] u16;
        for *hs it.* = 0xaaaa;
        sel: u64 = 0b1010_0000_0000_0101;
        pf, ph := fs.data, hs.data;
        #asm AVX512F {
            kmovw km:, sel;
            movups vf:, [pf];
            vcvtps2ph.z [ph] & km, vf, 0;
        }
        good := hs[0] == 0 && hs[1] == 0xaaaa && hs[2] == 0x4000 && hs[13] == 0x4a80 && hs[15] == 0x4b80 && hs[14] == 0xaaaa;
        expect("cvtps2ph masked store", cast(u64) good, 1);
        dst: [32] u16;
        for *dst it.* = 0x5555;
        pd := dst.data;
        #asm AVX512F {
            movdqu vd:, [pd];
            vcvtps2ph.z vd & km, vf, 0;
            movdqu [pd], vd;
        }
        good = dst[0] == 0 && dst[1] == 0x5555 && dst[2] == 0x4000 && dst[3] == 0x5555 && dst[15] == 0x4b80 && dst[16] == 0;
        expect("cvtps2ph merge mask", cast(u64) good, 1);
        #asm AVX512F {
            movdqu vd, [pd];
            vcvtps2ph.z vd &* km, vf, 0;
            movdqu [pd], vd;
        }
        expect("cvtps2ph zero mask", cast(u64) (dst[1] == 0 && dst[2] == 0x4000), 1);
        back: [16] float32;
        for *back it.* = -1;
        pb := back.data;
        #asm AVX512F {
            vcvtph2ps.z vb:, [ph];
            movups [pb], vb;
        }
        expect("cvtph2ps zmm", cast(u64) (back[2] == 2 && back[15] == 15 && back[1] == -1706.0 / 32768.0), 1);
    }
}

// ---- SHA ----

be32 :: (p: *u8) -> u32 {
    return ((cast(u32) p[0]) << 24) | ((cast(u32) p[1]) << 16) | ((cast(u32) p[2]) << 8) | p[3];
}

// Pad a message into 64-byte blocks of big-endian words.
pad_message :: (text: string) -> [..] u32 {
    bytes: [..] u8;
    for 0..text.count - 1 array_add(*bytes, text[it]);
    array_add(*bytes, 0x80);
    while bytes.count % 64 != 56 array_add(*bytes, 0);
    bit_count := cast(u64) text.count * 8;
    for < 0..7 array_add(*bytes, cast(u8) (bit_count >> (it * 8)));
    words: [..] u32;
    for i: 0..bytes.count / 4 - 1 array_add(*words, be32(*bytes[4 * i]));
    return words;
}

sha1_rounds :: (abcd: *u32, we: *u32, $func: int) {
    #asm SHA {
        movdqu.x s:, [abcd];
        movdqu.x w:, [we];
        sha1rnds4 s, w, func;
        movdqu.x [abcd], s;
    }
}

// SHA-1 vectors hold four words with the earliest in the top dword.
sha1 :: (text: string) -> [5] u32 {
    h := u32.[0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476, 0xc3d2_e1f0];
    words := pad_message(text);
    for block: 0..words.count / 16 - 1 {
        w: [20][4] u32;
        for g: 0..3 for j : 0..3 w[g][3 - j] = words[block * 16 + 4 * g + j];
        for g: 4..19 {
            m4, m3, m2, m1, dst := w[g - 4].data, w[g - 3].data, w[g - 2].data, w[g - 1].data, w[g].data;
            #asm SHA {
                movdqu.x t:, [m4];
                movdqu.x u:, [m3];
                sha1msg1 t, u;
                movdqu.x u, [m2];
                pxor.x t, u;
                movdqu.x u, [m1];
                sha1msg2 t, u;
                movdqu.x [dst], t;
            }
        }
        abcd := u32.[h[3], h[2], h[1], h[0]];
        prev: [4] u32;
        we: [4] u32;
        for g: 0..19 {
            pw, pp, pa, pe := w[g].data, prev.data, abcd.data, we.data;
            if g == 0 {
                e0 := u32.[0, 0, 0, h[4]];
                pz := e0.data;
                #asm { movdqu.x x:, [pz]; movdqu.x y:, [pw]; paddd.x x, y; movdqu.x [pe], x; }
            } else {
                // E for these rounds is A from four rounds back, rotated.
                #asm SHA { movdqu.x x:, [pp]; movdqu.x y:, [pw]; sha1nexte x, y; movdqu.x [pe], x; }
            }
            prev = abcd;
            if g < 5 sha1_rounds(pa, pe, 0);
            else if g < 10 sha1_rounds(pa, pe, 1);
            else if g < 15 sha1_rounds(pa, pe, 2);
            else sha1_rounds(pa, pe, 3);
        }
        zero: [4] u32;
        pp, pz, pe := prev.data, zero.data, we.data;
        #asm SHA { movdqu.x x:, [pp]; movdqu.x y:, [pz]; sha1nexte x, y; movdqu.x [pe], x; }
        h[0] += abcd[3];
        h[1] += abcd[2];
        h[2] += abcd[1];
        h[3] += abcd[0];
        h[4] += we[3];
    }
    return h;
}

SHA256_K :: u32.[
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

// SHA-256 message vectors hold four words with the earliest in the low dword; the
// state is split into ABEF and CDGH (A and C in the top dwords).
sha256 :: (text: string) -> [8] u32 {
    h := u32.[
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    k := SHA256_K;
    words := pad_message(text);
    for block: 0..words.count / 16 - 1 {
        w: [16][4] u32;
        for g: 0..3 for j : 0..3 w[g][j] = words[block * 16 + 4 * g + j];
        for g: 4..15 {
            m4, m3, m2, m1, dst := w[g - 4].data, w[g - 3].data, w[g - 2].data, w[g - 1].data, w[g].data;
            #asm SHA {
                movdqu.x t:, [m4];
                movdqu.x u:, [m3];
                sha256msg1 t, u;
                movdqu.x u, [m1];
                movdqu.x v:, [m2];
                palignr.x u, v, 4;
                paddd.x t, u;
                movdqu.x u, [m1];
                sha256msg2 t, u;
                movdqu.x [dst], t;
            }
        }
        abef := u32.[h[5], h[4], h[1], h[0]];
        cdgh := u32.[h[7], h[6], h[3], h[2]];
        for g: 0..15 {
            pw, pk, p1, p2 := w[g].data, *k[4 * g], abef.data, cdgh.data;
            #asm SHA {
                movdqu.x ab:, [p1];
                movdqu.x cd:, [p2];
                movdqu.x wk:, [pw];
                movdqu.x kk:, [pk];
                paddd.x wk, kk;
                sha256rnds2 cd, ab, wk;
                pshufd.x wk, wk, 0x0e;
                sha256rnds2 ab, cd, wk;
                movdqu.x [p1], ab;
                movdqu.x [p2], cd;
            }
        }
        h[0] += abef[3];
        h[1] += abef[2];
        h[4] += abef[1];
        h[5] += abef[0];
        h[2] += cdgh[3];
        h[3] += cdgh[2];
        h[6] += cdgh[1];
        h[7] += cdgh[0];
    }
    return h;
}

digest_matches :: (got: [] u32, want: [] u32) -> bool {
    if got.count != want.count return false;
    for got if it != want[it_index] return false;
    return true;
}

check_sha :: () {
    long :: "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    // FIPS 180-4 example digests.
    d := sha1("abc");
    expect("sha1 abc", cast(u64) digest_matches(d, u32.[0xa9993e36, 0x4706816a, 0xba3e2571, 0x7850c26c, 0x9cd0d89d]), 1);
    d = sha1(long);
    expect("sha1 two blocks", cast(u64) digest_matches(d, u32.[0x84983e44, 0x1c3bd26e, 0xbaae4aa1, 0xf95129e5, 0xe54670f1]), 1);
    d = sha1("");
    expect("sha1 empty", cast(u64) digest_matches(d, u32.[0xda39a3ee, 0x5e6b4b0d, 0x3255bfef, 0x95601890, 0xafd80709]), 1);
    e := sha256("abc");
    expect("sha256 abc", cast(u64) digest_matches(e, u32.[0xba7816bf, 0x8f01cfea, 0x414140de, 0x5dae2223, 0xb00361a3, 0x96177a9c, 0xb410ff61, 0xf20015ad]), 1);
    e = sha256(long);
    expect("sha256 two blocks", cast(u64) digest_matches(e, u32.[0x248d6a61, 0xd20638b8, 0xe5c02693, 0x0c3e6039, 0xa33ce459, 0x64ff2167, 0xf6ecedd4, 0x19db06c1]), 1);
    e = sha256("");
    expect("sha256 empty", cast(u64) digest_matches(e, u32.[0xe3b0c442, 0x98fc1c14, 0x9afbf4c8, 0x996fb924, 0x27ae41e4, 0x649b934c, 0xa495991b, 0x7852b855]), 1);

    // A legacy SHA instruction keeps the bytes above the xmm register.
    {
        wide: [8] u32;
        for *wide it.* = 0x0101_0101 * cast(u32) (it_index + 1);
        pw := wide.data;
        #asm SHA, AVX {
            movdqu.y r:, [pw];
            movdqu.y s:, [pw];
            sha1msg1 r, s;
            movdqu.y [pw], r;
        }
        expect("sha keeps upper", cast(u64) (wide[4] == 0x0505_0505 && wide[7] == 0x0808_0808), 1);
    }
}

// ---- GFNI ----

gf_reference_mul :: (a: u8, b: u8) -> u8 {
    product: u16 = 0;
    for 0..7 if (b >> it) & 1 product ^= (cast(u16) a) << it;
    for < 8..14 if (product >> it) & 1 product ^= cast(u16) 0x11b << (it - 8);
    return cast(u8) product;
}

gf_reference_affine :: (matrix: u64, x: u8, c: u8) -> u8 {
    r: u8 = 0;
    for i: 0..7 {
        row := cast(u8) (matrix >> ((7 - i) * 8));
        t := row & x;
        parity: u8 = 0;
        for 0..7 parity ^= (t >> it) & 1;
        r |= parity << i;
    }
    return r ^ c;
}

check_gfni :: () {
    // Every product, 32 at a time (VEX ymm).
    a: [32] u8;
    b: [32] u8;
    p: [32] u8;
    pa, pb, pp := a.data, b.data, p.data;
    wrong := 0;
    for x: 0..255 {
        for half: 0..7 {
            for j: 0..31 {
                a[j] = cast(u8) x;
                b[j] = cast(u8) (half * 32 + j);
            }
            #asm AVX, GFNI {
                movdqu va:, [pa];
                movdqu vb:, [pb];
                vgf2p8mulb vp:, va, vb;
                movdqu [pp], vp;
            }
            for j: 0..31 if p[j] != gf_reference_mul(a[j], b[j]) wrong += 1;
        }
    }
    expect("gf2p8mulb", cast(u64) wrong, 0);

    // Legacy form: the destination is the first factor.
    for j: 0..15 {
        a[j] = cast(u8) (j * 17 + 3);
        b[j] = cast(u8) (200 - j * 9);
    }
    #asm GFNI {
        movdqu.x r:, [pa];
        movdqu.x s:, [pb];
        gf2p8mulb r, s;
        movdqu.x [pp], r;
    }
    wrong = 0;
    for j: 0..15 if p[j] != gf_reference_mul(a[j], b[j]) wrong += 1;
    expect("gf2p8mulb legacy", cast(u64) wrong, 0);

    // Affine maps with pseudo-random matrices and constants.
    state: u64 = 0x9e37_79b9_7f4a_7c15;
    next :: () -> u64 #expand {
        `state ^= `state << 13;
        `state ^= `state >> 7;
        `state ^= `state << 17;
        return `state;
    }
    m: [4] u64;
    xs: [32] u8;
    pm, px := m.data, xs.data;
    wrong = 0;
    for 1..64 {
        for *m it.* = next();
        for *xs it.* = cast(u8) next();
        #asm AVX, GFNI {
            movdqu vx:, [px];
            movdqu vm:, [pm];
            vgf2p8affineqb vr:, vx, vm, 0x5a;
            movdqu [pp], vr;
        }
        for j: 0..31 if p[j] != gf_reference_affine(m[j / 8], xs[j], 0x5a) wrong += 1;
    }
    expect("gf2p8affineqb", cast(u64) wrong, 0);

    // The AES S-box is the inverse followed by the AES affine map.
    aes_matrix: u64 = 0xf1e3_c78f_1f3e_7cf8;
    identity: u64 = 0x0102_0408_1020_4080;
    sbox: [256] u8;
    inverse: [256] u8;
    all: [256] u8;
    for 0..255 all[it] = cast(u8) it;
    for chunk: 0..7 {
        src, dst, dst2 := *all[chunk * 32], *sbox[chunk * 32], *inverse[chunk * 32];
        pam, pid := *aes_matrix, *identity;
        #asm AVX, GFNI {
            movdqu vx:, [src];
            vgf2p8affineinvqb vs:, vx, [pam]!, 0x63;
            movdqu [dst], vs;
        }
        #asm GFNI {
            movdqu.x vx, [src];
            movq vi:, [pid];
            punpcklqdq.x vi, vi;
            gf2p8affineinvqb vx, vi, 0;
            movdqu.x [dst2], vx;
            movdqu.x vx, [src + 16];
            gf2p8affineinvqb vx, vi, 0;
            movdqu.x [dst2 + 16], vx;
        }
    }
    expect("sbox 00", sbox[0x00], 0x63);
    expect("sbox 01", sbox[0x01], 0x7c);
    expect("sbox 02", sbox[0x02], 0x77);
    expect("sbox 10", sbox[0x10], 0xca);
    expect("sbox 53", sbox[0x53], 0xed);
    expect("sbox 80", sbox[0x80], 0xcd);
    expect("sbox ff", sbox[0xff], 0x16);
    wrong = 0;
    for x: 1..255 if gf_reference_mul(cast(u8) x, inverse[x]) != 1 wrong += 1;
    expect("field inverse", cast(u64) (wrong + inverse[0]), 0);

    // EVEX: a merge-masked product and a zero-masked bit reversal (broadcast matrix).
    {
        za: [64] u8;
        zb: [64] u8;
        zr: [64] u8;
        for 0..63 {
            za[it] = cast(u8) (it * 5 + 1);
            zb[it] = cast(u8) (it * 11 + 7);
            zr[it] = 0xee;
        }
        sel: u64 = 0xf0f0_0000_0000_00ff;
        reverse: u64 = 0x8040_2010_0804_0201;
        pza, pzb, pzr, prev := za.data, zb.data, zr.data, *reverse;
        #asm AVX512F, AVX512BW, GFNI {
            kmovq km:, sel;
            movdqu8 va:, [pza];
            movdqu8 vb:, [pzb];
            movdqu8 vr:, [pzr];
            vgf2p8mulb vr & km, va, vb;
            movdqu8 [pzr], vr;
        }
        wrong = 0;
        for 0..63 {
            on := (sel >> it) & 1;
            want := ifx on != 0 then gf_reference_mul(za[it], zb[it]) else 0xee;
            if zr[it] != want wrong += 1;
        }
        expect("gf2p8mulb merge mask", cast(u64) wrong, 0);
        #asm AVX512F, AVX512BW, GFNI {
            vgf2p8affineqb vr &* km, va, [prev]!, 0;
            movdqu8 [pzr], vr;
        }
        wrong = 0;
        for 0..63 {
            on := (sel >> it) & 1;
            x := za[it];
            reversed: u8 = 0;
            for bit: 0..7 if (x >> bit) & 1 reversed |= cast(u8) (0x80 >> bit);
            want := ifx on != 0 then reversed else 0;
            if zr[it] != want wrong += 1;
        }
        expect("gf2p8affineqb broadcast zero mask", cast(u64) wrong, 0);
    }
}

main :: () {
    check_f16c();
    check_sha();
    check_gfni();
    if failures exit(1);
    print("ok\n");
}
