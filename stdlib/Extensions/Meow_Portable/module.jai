// Portable byte-oriented Meow hash for targets without AES-NI.
// @experimental

// Hashes a byte slice with a caller-supplied or default Meow seed.
// Hashes bytes with Meow 0.5, returning the low and high little-endian 64-bit lanes.
// This scalar implementation works without AES-NI; it is slower than `MeowHash`.
//
// ```jai
// digest := hash_bytes(cast([] u8) "abc");
// assert(digest[0] == 0x19f773ec430acc3b);
// assert(digest[1] == 0x5f6d236b424c9ab4);
// ```
hash_bytes :: (bytes: [] u8, seed: [128] u8 = Meow.MeowDefaultSeed) -> [2] u64 {
    B.assert(bytes.count >= 0 && (bytes.count == 0 || bytes.data != null));
    state: [8] Portable_Block;
    for 0..7 for j: 0..15 state[it].b[j] = seed[it * 16 + j];

    blocks := bytes.count / 256;
    for block: 0..blocks - 1 for lane : 0..7 {
        at := block * 256 + lane * 32;
        portable_mix(*state, lane, portable_load(bytes, at + 15), portable_load(bytes, at), portable_load(bytes, at + 1), portable_load(bytes, at + 16));
    }

    tail := blocks * 256;
    count := bytes.count - tail;
    remainder := count & 15;
    r1, r3: Portable_Block;

    for 0..remainder - 1 r1.b[it] = bytes[tail + (count & 0xF0) + it];

    if (count & 16) != 0 {
        r3 = r1;
        r1 = portable_load(bytes, tail + (count & 0xF0) - 16);
    }

    zero: Portable_Block;
    length := portable_length(cast(u64) bytes.count);

    portable_mix(*state, 0, portable_align(r1, r3, 15), r1, portable_align(r1, r3, 1), r3);
    portable_mix(*state, 1, portable_align(zero, length, 15), zero, portable_align(zero, length, 1), length);

    for 0..(count >> 5) - 1 {
        at := tail + it * 32;
        portable_mix(*state, (it + 2) % 8, portable_load(bytes, at + 15), portable_load(bytes, at), portable_load(bytes, at + 1), portable_load(bytes, at + 16));
    }

    for 0..11 portable_shuffle(*state, it % 8);

    portable_add(*state[0], state[2]);
    portable_add(*state[1], state[3]);
    portable_add(*state[4], state[6]);
    portable_add(*state[5], state[7]);
    portable_xor(*state[0], state[1]);
    portable_xor(*state[4], state[5]);
    portable_add(*state[0], state[4]);

    return .[portable_lane(state[0], 0), portable_lane(state[0], 8)];
}

#scope_file
B :: #import "Basic";
Meow :: #import "meow_hash";

Portable_Block :: struct {
    b: [16] u8;
}

portable_load :: (bytes: [] u8, at: int) -> Portable_Block {
    result: Portable_Block;
    for 0..15 result.b[it] = bytes[at + it];
    return result;
}

portable_lane :: (block: Portable_Block, at: int) -> u64 {
    value: u64;
    for 0..7 value |= (cast(u64) block.b[at + it]) << (it * 8);
    return value;
}

portable_length :: (value: u64) -> Portable_Block {
    result: Portable_Block;
    for 0..7 result.b[it] = cast(u8) ((value >> (it * 8)) & 255);
    return result;
}

portable_add :: (a: *Portable_Block, b: Portable_Block) #no_aoc {
    for lane: 0..1 {
        at := lane * 8;
        value := portable_lane(a.*, at) + portable_lane(b, at);
        for 0..7 a.b[at + it] = cast(u8) ((value >> (it * 8)) & 255);
    }
}

portable_xor :: (a: *Portable_Block, b: Portable_Block) {
    for 0..15 a.b[it] ^= b.b[it];
}

portable_align :: (high: Portable_Block, low: Portable_Block, offset: int) -> Portable_Block {
    result: Portable_Block;

    for 0..15 {
        at := it + offset;
        result.b[it] = ifx at < 16 then low.b[at] else high.b[at - 16];
    }

    return result;
}

portable_gf :: (a: u32, b: u32) -> u32 {
    x := a;
    y := b;
    result: u32;

    for 0..7 {
        if (y & 1) != 0 result ^= x;
        x <<= 1;
        if (x & 256) != 0 x ^= 0x11B;
        y >>= 1;
    }

    return result;
}

portable_sbox :: () -> [256] u8 {
    table: [256] u8;

    for n: 0..255 {
        // Inverse affine transform, followed by inversion in GF(2^8).
        x := cast(u32) n;
        value := (((x << 1) | (x >> 7)) ^ ((x << 3) | (x >> 5)) ^ ((x << 6) | (x >> 2)) ^ 5) & 255;
        inverse: u32 = 1;
        exponent := 254;

        while exponent > 0 {
            if (exponent & 1) != 0 inverse = portable_gf(inverse, value);
            value = portable_gf(value, value);
            exponent >>= 1;
        }

        table[n] = cast(u8) inverse;
    }

    return table;
}

PORTABLE_INVERSE_SBOX :: #run portable_sbox();

portable_aesdec :: (a: *Portable_Block, key: Portable_Block) {
    shifted: Portable_Block;

    for col: 0..3 for row : 0..3 {
        shifted.b[col * 4 + row] = PORTABLE_INVERSE_SBOX[a.b[((col - row + 4) % 4) * 4 + row]];
    }

    for col: 0..3 for row : 0..3 {
        at := col * 4;
        // Inverse MixColumns coefficients rotated for each output row.
        a.b[at + row] = cast(u8) (
            portable_gf(shifted.b[at + row], 14) ^ portable_gf(shifted.b[at + (row + 1) % 4], 11) ^
            portable_gf(shifted.b[at + (row + 2) % 4], 13) ^ portable_gf(shifted.b[at + (row + 3) % 4], 9));
    }

    portable_xor(a, key);
}

portable_mix :: (s: *[8] Portable_Block, lane: int, a: Portable_Block, b: Portable_Block, c: Portable_Block, d: Portable_Block) {
    r1 := lane;
    r2 := (lane + 4) % 8;
    r3 := (lane + 6) % 8;
    r4 := (lane + 1) % 8;
    r5 := (lane + 2) % 8;

    portable_aesdec(*s.*[r1], s.*[r2]);
    portable_add(*s.*[r3], a);
    portable_xor(*s.*[r2], b);
    portable_aesdec(*s.*[r2], s.*[r4]);
    portable_add(*s.*[r5], c);
    portable_xor(*s.*[r4], d);
}

portable_shuffle :: (s: *[8] Portable_Block, lane: int) {
    a := lane;
    b := (lane + 1) % 8;
    c := (lane + 2) % 8;
    d := (lane + 4) % 8;
    e := (lane + 5) % 8;
    f := (lane + 6) % 8;

    portable_aesdec(*s.*[a], s.*[d]);
    portable_add(*s.*[b], s.*[e]);
    portable_xor(*s.*[d], s.*[f]);
    portable_aesdec(*s.*[d], s.*[b]);
    portable_add(*s.*[e], s.*[f]);
    portable_xor(*s.*[b], s.*[c]);
}
