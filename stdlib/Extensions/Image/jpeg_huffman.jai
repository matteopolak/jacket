// Huffman tables and the bit reader of the JPEG decoder.
//
// The bit buffer is left aligned: the next bit of the stream is bit 63 of `bits` and `nbits`
// counts the valid bits. Block decoders keep bits, nbits, pos, data and end in locals and use the
// jpeg_ macros below on them. After a marker, or at the end of the data, the reader supplies zero
// bits and counts them in `pad`; `pad > nbits` therefore means that padding has been consumed.

Jpeg_Huffman :: struct {
    defined: bool;
    // (length << 8) | symbol for every 9-bit prefix of a code of at most 9 bits, else 0.
    fast: [512] u16;
    // For AC tables, per 9-bit prefix: the whole run/size code plus its extra bits when they fit
    // in 9 bits. Low byte: bits used. Next byte: run, or 255 for end of block. High half: the
    // coefficient. 0 when the prefix needs the general path.
    ac_fast: [512] u32;
    limit: [17] s32; // one past the last code of each length, left-aligned to 16 bits
    first_code: [17] s32;
    first_index: [17] s32;
    symbols: [256] u8;
}

JPEG_ZIGZAG :: u8.[
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5,
    12, 19, 26, 33, 40, 48, 41, 34, 27, 20, 13, 6, 7, 14, 21, 28,
    35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51,
    58, 59, 52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

// Sign-extends a `size`-bit magnitude read from the stream.
jpeg_extend :: inline (value: s32, size: s32) -> s32 {
    if value < (1 << (size - 1)) return value - (1 << size) + 1;
    return value;
}

jpeg_build_huffman :: (h: *Jpeg_Huffman, counts: *u8, symbols: *u8, is_ac: bool) -> bool {
    memset(h, 0, size_of(Jpeg_Huffman));

    total: s32 = 0;
    for 0..15 total += counts[it];

    for 0..total - 1 {
        h.symbols[it] = symbols[it];
        if !is_ac && symbols[it] > 15 return false;
    }

    code: s32 = 0;
    index: s32 = 0;

    for length: 1..16 {
        n := cast(s32) counts[length - 1];
        if code + n > (1 << length) return false;

        h.first_code[length] = code;
        h.first_index[length] = index;

        if length <= 9 {
            for i: 0..n - 1 {
                start := (code + i) << (9 - length);

                for j: 0..(1 << (9 - length)) - 1 {
                    h.fast[start + j] = cast,trunc(u16) ((length << 8) | h.symbols[index + i]);
                }
            }
        }

        code += n;
        index += n;
        h.limit[length] = code << (16 - length);
        code <<= 1;
    }

    if is_ac {
        for index: 0..511 {
            prefix := cast(s32) index;
            entry := cast(s32) h.fast[index];

            if entry == 0 continue;

            length := entry >> 8;
            symbol := entry & 255;
            run := symbol >> 4;
            size := symbol & 15;

            if size == 0 {
                if symbol == 0 {
                    h.ac_fast[index] = cast,trunc(u32) (length | (255 << 8));
                } else if symbol == 0xF0 {
                    h.ac_fast[index] = cast,trunc(u32) (length | (15 << 8));
                }
            } else if length + size <= 9 {
                extra := (prefix >> (9 - length - size)) & ((1 << size) - 1);
                value := jpeg_extend(extra, size);
                h.ac_fast[index] = cast,trunc(u32) ((value << 16) | (run << 8) | (length + size));
            }
        }
    }

    h.defined = true;

    return true;
}

// Decodes a code longer than 9 bits (or reports a bad code as symbol 0). `bits` is the
// left-aligned bit buffer.
jpeg_huffman_long :: (h: *Jpeg_Huffman, bits: u64) -> symbol: s32, length: s32 #no_abc {
    window := cast(s32) (bits >> 48);

    for step: 10..16 {
        length := cast(s32) step;
        if window < h.limit[length] {
            index := (window >> (16 - length)) - h.first_code[length] + h.first_index[length];
            return h.symbols[index & 255], length;
        }
    }

    return 0, 16;
}

jpeg_decode_symbol :: (h: *Jpeg_Huffman) -> s32 #expand #no_abc {
    entry := cast(s32) h.fast[`bits >> 55];
    if entry != 0 {
        length := entry >> 8;
        `bits <<= cast(u64) length;
        `nbits -= length;
        return entry & 255;
    }

    symbol, length := jpeg_huffman_long(h, `bits);

    `bits <<= cast(u64) length;
    `nbits -= length;

    return symbol;
}

// Reads n bits, 1 <= n <= 16. At least n bits must be buffered.
jpeg_get_bits :: (n: s32) -> s32 #expand #no_abc {
    value := cast(s32) (`bits >> cast(u64) (64 - n));
    `bits <<= cast(u64) n;
    `nbits -= n;
    return value;
}

// Tops the buffer up to at least 32 bits (one code plus its extra bits is at most 31).
jpeg_refill :: () #expand #no_abc {
    if `nbits < 32 {
        done := false;

        if `pos + 8 <= `end {
            p := `data + `pos;
            w := ((cast(u64) p[0]) << 56) | ((cast(u64) p[1]) << 48) | ((cast(u64) p[2]) << 40) | ((cast(u64) p[3]) << 32) | ((cast(u64) p[4]) << 24) | ((cast(u64) p[5]) << 16) | ((cast(u64) p[6]) << 8) | (cast(u64) p[7]);
            inverse := ~w;

            if ((inverse - 0x0101010101010101) & ~inverse & 0x8080808080808080) == 0 {
                // None of the eight bytes is 0xFF: take as many whole bytes as fit.
                n := (64 - `nbits) >> 3;
                shift := cast(u64) (64 - n * 8);

                `bits |= ((w >> shift) << shift) >> cast(u64) `nbits;
                `nbits += n * 8;
                `pos += n;
                done = true;
            }
        }

        if !done {
            `d.bits = `bits;
            `d.nbits = `nbits;
            `d.pos = `pos;
            jpeg_refill_slow(`d);
            `bits = `d.bits;
            `nbits = `d.nbits;
            `pos = `d.pos;
        }
    }
}

// Byte-at-a-time refill that unstuffs 0xFF00, stops at markers and pads with zero bits.
jpeg_refill_slow :: (d: *Jpeg_Decoder) #no_abc {
    while d.nbits <= 56 {
        if d.marker || d.pos >= d.end {
            d.pad += 8;
            d.nbits += 8;
            continue;
        }

        byte := d.data[d.pos];
        if byte == 0xFF {
            if d.pos + 1 >= d.end {
                d.marker = 0xFF;
                continue;
            }

            next := d.data[d.pos + 1];
            if next == 0xFF {
                d.pos += 1;
                continue;
            }

            if next != 0 {
                d.marker = next;
                continue;
            }

            d.pos += 1;
        }

        d.bits |= (cast(u64) byte) << cast(u64) (56 - d.nbits);
        d.nbits += 8;
        d.pos += 1;
    }
}

// Called between restart intervals: drops the buffered bits and finds the next RSTn marker.
// `expected` is the number (0..7) the next restart marker should carry; `skipped` returns how
// many whole intervals were lost when the marker found is further ahead. False when the entropy
// data has no further restart marker (the scan is over).
jpeg_sync_restart :: (d: *Jpeg_Decoder, expected: *s32, skipped: *s32) -> bool {
    d.bits = 0;
    d.nbits = 0;
    d.pad = 0;
    skipped.* = 0;

    while true {
        code: u8 = d.marker;
        if code == 0 {
            while d.pos + 1 < d.end {
                if d.data[d.pos] == 0xFF && d.data[d.pos + 1] != 0 && d.data[d.pos + 1] != 0xFF {
                    code = d.data[d.pos + 1];
                    break;
                }

                d.pos += 1;
            }

            if code == 0 {
                d.pos = d.end;
                return false;
            }
        }

        if code < 0xD0 || code > 0xD7 {
            d.marker = 0;
            return false;
        }

        d.marker = 0;
        d.pos += 2;
        number := cast(s32) code - 0xD0;
        ahead := (number - expected.*) & 7;

        expected.* = (number + 1) & 7;

        if ahead <= 2 {
            skipped.* = ahead;
            return true;
        }
        // An old marker from before the damage: look for the next one.
    }

    return false;
}
