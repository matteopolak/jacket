#scope_module

// Base values and extra-bit counts of the length symbols 257..285 and the distance symbols 0..29.
LENGTH_BASE :: u16.[3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
LENGTH_EXTRA :: u8.[0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
DISTANCE_BASE :: u16.[1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577];
DISTANCE_EXTRA :: u8.[0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];

// The order in which a dynamic block lists its code length code lengths.
CODE_LENGTH_ORDER :: u8.[16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

// Little-endian and big-endian integers from bytes.
le16 :: inline (p: *u8) -> u32 {
    a: u32 = p[0];
    b: u32 = p[1];

    return a | (b << 8);
}

le32 :: inline (p: *u8) -> u32 {
    return le16(p) | (le16(p + 2) << 16);
}

be32 :: inline (p: *u8) -> u32 {
    a: u32 = p[0];
    b: u32 = p[1];
    c: u32 = p[2];
    d: u32 = p[3];

    return (a << 24) | (b << 16) | (c << 8) | d;
}

reverse_bits :: inline (code: u32, length: u32) -> u32 {
    v := code;
    v = ((v >> 1) & 0x5555) | ((v & 0x5555) << 1);
    v = ((v >> 2) & 0x3333) | ((v & 0x3333) << 2);
    v = ((v >> 4) & 0x0F0F) | ((v & 0x0F0F) << 4);
    v = ((v >> 8) & 0x00FF) | ((v & 0x00FF) << 8);

    return v >> (16 - length);
}
