md5 :: (input: string) -> string #no_aoc {
    length := ((input.count + 9 + 63) / 64) * 64;
    buffer := alloc_string(length);

    defer free(buffer);
    memset(buffer.data, 0, length);
    memcpy(buffer.data, input.data, input.count);
    buffer.data[input.count] = 128;
    bit_count := (cast(u64) input.count) * 8;
    for i: 0..7 buffer.data[length - 8 + i] = cast,truncate(u8) (bit_count >> (i * 8));

    state: [4] u32 = .[0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];

    for block: 0..length / 64 - 1 {
        words: [16] u32;

        for i: 0..15 {
            offset := block * 64 + i * 4;
            word: u32;
            for byte: 0..3 word |= (cast(u32) buffer.data[offset + byte]) << (byte * 8);
            words[i] = word;
        }

        a := state[0];
        b := state[1];
        c := state[2];
        d := state[3];

        for i: 0..63 {
            f: u32;
            g: s64;

            if i < 16 {
                f = (b & c) | (~b & d);
                g = i;
            } else if i < 32 {
                f = (d & b) | (~d & c);
                g = (5 * i + 1) % 16;
            } else if i < 48 {
                f = b ^ c ^ d;
                g = (3 * i + 5) % 16;
            } else {
                f = c ^ (b | ~d);
                g = (7 * i) % 16;
            }

            value := a + f + K[i] + words[g];
            shift := shifts[i];
            next := b + ((value << shift) | (value >> (32 - shift)));

            a = d;
            d = c;
            c = b;
            b = next;
        }

        state[0] += a;
        state[1] += b;
        state[2] += c;
        state[3] += d;
    }

    result := alloc_string(32);

    for i: 0..15 {
        byte := (state[i / 4] >> ((i % 4) * 8)) & 255;
        result.data[i * 2] = "0123456789abcdef"[byte >> 4];
        result.data[i * 2 + 1] = "0123456789abcdef"[byte & 15];
    }

    return result;
}

#scope_file
#import "Basic";

shifts: [64] u32 : .[
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
    5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
    4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
    6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21
];

K: [64] u32 : .[
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x2441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x4881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391
];
