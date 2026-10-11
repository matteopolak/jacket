// meow_hash: MeowHash fills the caller's `#asm` vector register, MeowU64From reads a lane of it.
// The expected values come from a plain software model written here (AES decryption round,
// lane-wise adds and xors), so the test checks the module's `aesdec`/`paddq`/`pxor` pipeline
// against an independent implementation of the same mixing schedule. On CPUs other than x64 the
// module has no hash, so the program only prints ok.
#import "Basic";
#import "meow_hash";

#if CPU == .X64 {
    inv_sbox: [256] u8;

    gf_mul :: (a: u8, b: u8) -> u8 {
        x: u32 = a;
        y: u32 = b;
        r: u32;
        while y != 0 {
            if y & 1 r ^= x;
            x <<= 1;
            if x & 0x100 x ^= 0x11b;
            y >>= 1;
        }
        return cast,trunc(u8) r;
    }

    rotl8 :: (v: u32, n: u32) -> u32 {
        return ((v << n) | (v >> (8 - n))) & 0xff;
    }

    build_tables :: () {
        for x: 0..255 {
            inverse: u8;
            if x != 0 {
                for y: 1..255 if gf_mul(cast,trunc(u8) x, cast,trunc(u8) y) == 1 inverse = cast,trunc(u8) y;
            }
            v: u32 = inverse;
            s := v ^ rotl8(v, 1) ^ rotl8(v, 2) ^ rotl8(v, 3) ^ rotl8(v, 4) ^ 0x63;
            inv_sbox[s] = cast,trunc(u8) x;
        }
    }

    Lane :: [16] u8;

    // AESDEC: InvShiftRows, InvSubBytes, InvMixColumns, then xor with the round key.
    soft_aesdec :: (state: *Lane, key: Lane) {
        shifted: Lane;
        for r: 0..3 for c : 0..3 shifted[r + 4 * ((c + r) % 4)] = inv_sbox[state.*[r + 4 * c]];
        for c: 0..3 {
            a0 := shifted[4 * c];
            a1 := shifted[4 * c + 1];
            a2 := shifted[4 * c + 2];
            a3 := shifted[4 * c + 3];
            state.*[4 * c] = gf_mul(a0, 14) ^ gf_mul(a1, 11) ^ gf_mul(a2, 13) ^ gf_mul(a3, 9);
            state.*[4 * c + 1] = gf_mul(a0, 9) ^ gf_mul(a1, 14) ^ gf_mul(a2, 11) ^ gf_mul(a3, 13);
            state.*[4 * c + 2] = gf_mul(a0, 13) ^ gf_mul(a1, 9) ^ gf_mul(a2, 14) ^ gf_mul(a3, 11);
            state.*[4 * c + 3] = gf_mul(a0, 11) ^ gf_mul(a1, 13) ^ gf_mul(a2, 9) ^ gf_mul(a3, 14);
        }
        for 0..15 state.*[it] ^= key[it];
    }

    soft_add :: (a: *Lane, b: Lane) {
        for half: 0..1 {
            carry: u32;
            for i: 0..7 {
                sum: u32 = cast(u32) a.*[half * 8 + i] + cast(u32) b[half * 8 + i] + carry;
                a.*[half * 8 + i] = cast,trunc(u8) sum;
                carry = sum >> 8;
            }
        }
    }

    soft_xor :: (a: *Lane, b: Lane) {
        for 0..15 a.*[it] ^= b[it];
    }

    load_lane :: (p: *u8, offset: int) -> Lane {
        l: Lane;
        for 0..15 l[it] = p[offset + it];
        return l;
    }

    Lanes :: [8] Lane;

    soft_stage :: (x: *Lanes, i: int, p: *u8, offset: int) {
        r1 := i;
        r2 := (i + 4) % 8;
        r3 := (i + 6) % 8;
        r4 := (i + 1) % 8;
        r5 := (i + 2) % 8;
        soft_aesdec(*x.*[r1], x.*[r2]);
        soft_add(*x.*[r3], load_lane(p, offset + 15));
        soft_xor(*x.*[r2], load_lane(p, offset));
        soft_aesdec(*x.*[r2], x.*[r4]);
        soft_add(*x.*[r5], load_lane(p, offset + 1));
        soft_xor(*x.*[r4], load_lane(p, offset + 16));
    }

    soft_block :: (x: *Lanes, p: *u8) {
        for i: 0..7 soft_stage(x, i, p, 32 * i);
    }

    soft_shuffle :: (x: *Lanes, i: int) {
        a := i % 8;
        b := (i + 1) % 8;
        c := (i + 2) % 8;
        d := (i + 4) % 8;
        e := (i + 5) % 8;
        f := (i + 6) % 8;
        soft_aesdec(*x.*[a], x.*[d]);
        soft_add(*x.*[b], x.*[e]);
        soft_xor(*x.*[d], x.*[f]);
        soft_aesdec(*x.*[d], x.*[b]);
        soft_add(*x.*[e], x.*[f]);
        soft_xor(*x.*[b], x.*[c]);
    }

    soft_hash :: (seed: [128] u8, bytes: [] u8) -> Lane {
        x: Lanes;
        for i: 0..7 x[i] = load_lane(seed.data, 16 * i);
        p := bytes.data;
        remaining := bytes.count;
        while remaining >= 256 {
            soft_block(*x, p);
            p += 256;
            remaining -= 256;
        }
        if remaining > 0 {
            padded: [256] u8;
            for 0..remaining - 1 padded[it] = p[it];
            soft_block(*x, padded.data);
        }
        len: u64 = xx bytes.count;
        lengths: [2] u64;
        lengths[0] = len;
        lengths[1] = ~len;
        m := load_lane(cast(*u8) lengths.data, 0);
        soft_add(*x[0], m);
        soft_xor(*x[5], m);
        for i: 0..11 soft_shuffle(*x, i);
        soft_add(*x[0], x[2]);
        soft_add(*x[1], x[3]);
        soft_add(*x[4], x[6]);
        soft_add(*x[5], x[7]);
        soft_xor(*x[0], x[1]);
        soft_xor(*x[4], x[5]);
        soft_add(*x[0], x[4]);
        return x[0];
    }

    lane_u64 :: (l: Lane, index: int) -> u64 {
        v: u64;
        for i: 0..7 v |= (cast(u64) l[index * 8 + i]) << cast(u64) (8 * i);
        return v;
    }

    failures := 0;

    check_length :: (count: int) {
        data := NewArray(count + 1, u8);
        defer free(data.data);
        for 0..count - 1 data[it] = cast,trunc(u8) (it * 31 + count);
        bytes: [] u8 = .{ count, data.data };

        want := soft_hash(MeowDefaultSeed, bytes);
        #asm { hash: vec; }
        MeowHash(MeowDefaultSeed, xx bytes.count, bytes.data, hash);
        got0 := MeowU64From(hash, 0);
        got1 := MeowU64From(hash, 1);
        if got0 != lane_u64(want, 0) || got1 != lane_u64(want, 1) {
            print("FAIL length %: got %/%, want %/%\n", count, got0, got1, lane_u64(want, 0), lane_u64(want, 1));
            failures += 1;
        }
    }
}

main :: () {
    #if CPU == .X64 {
        build_tables();
        // The software AES round must itself match the FIPS-197 inverse S-box corners.
        if inv_sbox[0x63] != 0x00 || inv_sbox[0x00] != 0x52 {
            print("FAIL inverse S-box\n");
            failures += 1;
        }
        for count: s64.[0, 1, 15, 16, 31, 32, 33, 100, 255, 256, 257, 300, 511, 512, 700] check_length(count);

        // Equal input hashes equal; one differing byte or length changes the result.
        a := u8.[1, 2, 3, 4, 5];
        b := u8.[1, 2, 3, 4, 6];
        c := u8.[1, 2, 3, 4, 5, 0];
        #asm { ha: vec; }
        MeowHash(MeowDefaultSeed, 5, a.data, ha);
        #asm { hb: vec; }
        MeowHash(MeowDefaultSeed, 5, b.data, hb);
        #asm { hc: vec; }
        MeowHash(MeowDefaultSeed, 6, c.data, hc);
        #asm { hd: vec; }
        MeowHash(MeowDefaultSeed, 5, a.data, hd);
        if MeowU64From(ha, 0) != MeowU64From(hd, 0) || MeowU64From(ha, 1) != MeowU64From(hd, 1) {
            print("FAIL not deterministic\n");
            failures += 1;
        }
        if MeowU64From(ha, 0) == MeowU64From(hb, 0) {
            print("FAIL byte change\n");
            failures += 1;
        }
        if MeowU64From(ha, 0) == MeowU64From(hc, 0) {
            print("FAIL length change\n");
            failures += 1;
        }
        if failures != 0 exit(1);
    }
    print("ok\n");
}
