sha1 :: (s: string) -> [SHA_DIGEST_LENGTH] u8 {
    state := u32.[0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];
    if !hash_blocks(s, state.data, compress_sha1) return .{};

    result: [SHA_DIGEST_LENGTH] u8;
    for index: 0..4 write_word(result.data + index * 4, state[index]);

    return result;
}

sha256 :: (s: string) -> [SHA256_DIGEST_LENGTH] u8 {
    state := u32.[0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    if !hash_blocks(s, state.data, compress_sha256) return .{};

    result: [SHA256_DIGEST_LENGTH] u8;
    for index: 0..7 write_word(result.data + index * 4, state[index]);

    return result;
}

sha512 :: (s: string) -> [64] u8 {
    state := SHA512_START;
    if !hash_blocks_wide(s, state.data) return .{};

    result: [64] u8;
    for index: 0..7 write_wide(result.data + index * 8, state[index]);

    return result;
}

sha384 :: (s: string) -> [48] u8 {
    state := SHA384_START;
    if !hash_blocks_wide(s, state.data) return .{};

    result: [48] u8;
    for index: 0..5 write_wide(result.data + index * 8, state[index]);

    return result;
}

// State after hashing exactly one 64-byte block, so a fixed prefix can be reused.
sha256_block_state :: (block: *u8) -> [8] u32 {
    state := u32.[0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    compress_sha256(state.data, block);
    return state;
}

// Finishes a hash from `state`, which has already absorbed `prefix_bytes`
// (a multiple of 64) bytes, over the rest of the message `s`.
sha256_continue :: (state: [8] u32, prefix_bytes: int, s: string) -> [SHA256_DIGEST_LENGTH] u8 {
    working := state;
    if !hash_blocks(s, working.data, compress_sha256, prefix_bytes) return .{};

    result: [SHA256_DIGEST_LENGTH] u8;
    for index: 0..7 write_word(result.data + index * 4, working[index]);

    return result;
}

#scope_file
B :: #import "Basic";

SHA_DIGEST_LENGTH :: 20;
SHA256_DIGEST_LENGTH :: 32;
Compression :: #type (state: *u32, block: *u8);

hash_blocks :: (s: string, state: *u32, compress: Compression, prefix := 0) -> bool {
    if s.count < 0 || s.count > 0x1fffffffffffffff {
        context.logger("SHA input exceeds the 64-bit bit-length domain\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        B.assert(false, "SHA input exceeds the 64-bit bit-length domain");
        return false;
    }

    blocks := s.count / 64;
    for index: 0..blocks - 1 compress(state, s.data + index * 64);

    tail: [64] u8;
    remaining := s.count % 64;

    if remaining > 0 memcpy(tail.data, s.data + blocks * 64, remaining);

    tail[remaining] = 128;

    if remaining >= 56 {
        compress(state, tail.data);
        memset(tail.data, 0, 64);
    }

    bit_count: u64 = cast(u64) (s.count + prefix);
    bit_count *= 8;

    for index: 0..7 {
        tail[63 - index] = cast(u8) (bit_count & 255);
        bit_count >>= 8;
    }

    compress(state, tail.data);

    return true;
}

// SHA-384 and SHA-512 work on 128-byte blocks with a 128-bit bit count.
hash_blocks_wide :: (s: string, state: *u64) -> bool {
    if s.count < 0 || s.count > 0x0fffffffffffffff {
        context.logger("SHA input exceeds the length domain\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        B.assert(false, "SHA input exceeds the length domain");
        return false;
    }

    blocks := s.count / 128;
    for index: 0..blocks - 1 compress_sha512(state, s.data + index * 128);

    tail: [128] u8;
    remaining := s.count % 128;

    if remaining > 0 memcpy(tail.data, s.data + blocks * 128, remaining);

    tail[remaining] = 128;

    if remaining >= 112 {
        compress_sha512(state, tail.data);
        memset(tail.data, 0, 128);
    }

    // Only the low 64 bits of the 128-bit count can be non-zero here.
    bit_count: u64 = cast(u64) s.count;
    bit_count *= 8;

    for index: 0..7 {
        tail[127 - index] = cast(u8) (bit_count & 255);
        bit_count >>= 8;
    }

    compress_sha512(state, tail.data);

    return true;
}

write_wide :: (out: *u8, value: u64) {
    write_word(out, cast(u32) (value >> 32));
    write_word(out + 4, cast(u32) (value & 0xffffffff));
}

rotate_right_wide :: (value: u64, $amount: u64) -> u64 {
    #assert amount > 0 && amount < 64;
    return (value >> amount) | (value << (64 - amount));
}

compress_sha512 :: (state: *u64, block: *u8) #no_aoc {
    words: [80] u64;

    for index: 0..15 {
        high: u64 = read_word(block + index * 8);
        low: u64 = read_word(block + index * 8 + 4);
        words[index] = (high << 32) | low;
    }

    for index: 16..79 {
        prior := words[index - 15];
        recent := words[index - 2];
        small0 := rotate_right_wide(prior, 1) ^ rotate_right_wide(prior, 8) ^ (prior >> 7);
        small1 := rotate_right_wide(recent, 19) ^ rotate_right_wide(recent, 61) ^ (recent >> 6);

        words[index] = words[index - 16] + small0 + words[index - 7] + small1;
    }

    a := state[0];
    b := state[1];
    c := state[2];
    d := state[3];
    e := state[4];
    f := state[5];
    g := state[6];
    h := state[7];

    for index: 0..79 {
        big1 := rotate_right_wide(e, 14) ^ rotate_right_wide(e, 18) ^ rotate_right_wide(e, 41);
        choice := (e & f) ^ (~e & g);
        first := h + big1 + choice + SHA512_CONSTANTS[index] + words[index];
        big0 := rotate_right_wide(a, 28) ^ rotate_right_wide(a, 34) ^ rotate_right_wide(a, 39);
        majority := (a & b) ^ (a & c) ^ (b & c);
        second := big0 + majority;

        h = g;
        g = f;
        f = e;
        e = d + first;
        d = c;
        c = b;
        b = a;
        a = first + second;
    }

    state[0] += a;
    state[1] += b;
    state[2] += c;
    state[3] += d;
    state[4] += e;
    state[5] += f;
    state[6] += g;
    state[7] += h;
}

read_word :: (bytes: *u8) -> u32 {
    a: u32 = bytes[0];
    b: u32 = bytes[1];
    c: u32 = bytes[2];
    d: u32 = bytes[3];

    return (a << 24) | (b << 16) | (c << 8) | d;
}

write_word :: (out: *u8, value: u32) {
    out[0] = cast(u8) ((value >> 24) & 255);
    out[1] = cast(u8) ((value >> 16) & 255);
    out[2] = cast(u8) ((value >> 8) & 255);
    out[3] = cast(u8) (value & 255);
}

rotate_right :: inline (value: u32, $amount: u32) -> u32 {
    #assert amount > 0 && amount < 32;
    return (value >> amount) | (value << (32 - amount));
}

compress_sha1 :: (state: *u32, block: *u8) #no_aoc {
    words: [80] u32;
    for index: 0..15 words[index] = read_word(block + index * 4);

    for index: 16..79 {
        mixed := words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16];
        words[index] = rotate_right(mixed, 31);
    }

    a := state[0];
    b := state[1];
    c := state[2];
    d := state[3];
    e := state[4];

    for index: 0..79 {
        function: u32;
        constant: u32;

        if index < 20 {
            function = (b & c) | (~b & d);
            constant = 0x5a827999;
        } else if index < 40 {
            function = b ^ c ^ d;
            constant = 0x6ed9eba1;
        } else if index < 60 {
            function = (b & c) | (b & d) | (c & d);
            constant = 0x8f1bbcdc;
        } else {
            function = b ^ c ^ d;
            constant = 0xca62c1d6;
        }

        next := rotate_right(a, 27) + function + e + constant + words[index];

        e = d;
        d = c;
        c = rotate_right(b, 2);
        b = a;
        a = next;
    }

    state[0] += a;
    state[1] += b;
    state[2] += c;
    state[3] += d;
    state[4] += e;
}

compress_sha256 :: (state: *u32, block: *u8) #no_aoc {
    words: [64] u32;
    for index: 0..15 words[index] = read_word(block + index * 4);

    for index: 16..63 {
        prior := words[index - 15];
        recent := words[index - 2];
        small0 := ((prior >> 7) | (prior << 25)) ^ ((prior >> 18) | (prior << 14)) ^ (prior >> 3);
        small1 := ((recent >> 17) | (recent << 15)) ^ ((recent >> 19) | (recent << 13)) ^ (recent >> 10);

        words[index] = words[index - 16] + small0 + words[index - 7] + small1;
    }

    a := state[0];
    b := state[1];
    c := state[2];
    d := state[3];
    e := state[4];
    f := state[5];
    g := state[6];
    h := state[7];

    for index: 0..63 {
        big1 := ((e >> 6) | (e << 26)) ^ ((e >> 11) | (e << 21)) ^ ((e >> 25) | (e << 7));
        choice := (e & f) ^ (~e & g);
        first := h + big1 + choice + SHA256_CONSTANTS[index] + words[index];
        big0 := ((a >> 2) | (a << 30)) ^ ((a >> 13) | (a << 19)) ^ ((a >> 22) | (a << 10));
        majority := (a & b) ^ (a & c) ^ (b & c);
        second := big0 + majority;

        h = g;
        g = f;
        f = e;
        e = d + first;
        d = c;
        c = b;
        b = a;
        a = first + second;
    }

    state[0] += a;
    state[1] += b;
    state[2] += c;
    state[3] += d;
    state[4] += e;
    state[5] += f;
    state[6] += g;
    state[7] += h;
}

// Standard mathematical SHA-256 round constants, not implementation bodies.
SHA256_CONSTANTS: [64] u32 : .[
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
];

// Standard mathematical SHA-512 constants (cube roots of the first 80 primes).
SHA512_CONSTANTS: [80] u64 : .[
    0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc,
    0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118,
    0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694,
    0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65,
    0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
    0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70,
    0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
    0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
    0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30,
    0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
    0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b,
    0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178,
    0x06f067aa72176fba, 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
    0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817
];

SHA512_START: [8] u64 : .[
    0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
    0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b, 0x5be0cd19137e2179
];

SHA384_START: [8] u64 : .[
    0xcbbb9d5dc1059ed8, 0x629a292a367cd507, 0x9159015a3070dd17, 0x152fecd8f70e5939,
    0x67332667ffc00b31, 0x8eb44a8768581511, 0xdb0c2e0d64f98fa7, 0x47b5481dbefa4fa4
];
