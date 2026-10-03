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

#scope_file
B :: #import "Basic";

SHA_DIGEST_LENGTH :: 20;
SHA256_DIGEST_LENGTH :: 32;
Compression :: #type (state: *u32, block: *u8);

hash_blocks :: (s: string, state: *u32, compress: Compression) -> bool {
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

    bit_count: u64 = cast(u64) s.count;
    bit_count *= 8;

    for index: 0..7 {
        tail[63 - index] = cast(u8) (bit_count & 255);
        bit_count >>= 8;
    }

    compress(state, tail.data);

    return true;
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

rotate_right :: (value: u32, $amount: u32) -> u32 {
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
        small0 := rotate_right(prior, 7) ^ rotate_right(prior, 18) ^ (prior >> 3);
        small1 := rotate_right(recent, 17) ^ rotate_right(recent, 19) ^ (recent >> 10);

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
        big1 := rotate_right(e, 6) ^ rotate_right(e, 11) ^ rotate_right(e, 25);
        choice := (e & f) ^ (~e & g);
        first := h + big1 + choice + SHA256_CONSTANTS[index] + words[index];
        big0 := rotate_right(a, 2) ^ rotate_right(a, 13) ^ rotate_right(a, 22);
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
