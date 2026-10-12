// Stage 1: a string index of the text, built 64 bytes at a time with vector compares (the
// approach of simdjson and simd-json for finding which bytes are inside strings, written for
// this repo).
//
// The index lists, in order, the offset of every
//   - opening quote of a string,
//   - backslash inside a string (marked BACKSLASH_FLAG),
//   - closing quote of a string (marked CLOSE_FLAG).
// The reader (stage 2) takes a string's extent from it without scanning: a string whose opening
// quote is followed directly by its closing quote has no escapes, and stage 1 has already
// checked its bytes for control characters and invalid UTF-8.
//
// How: eight bit masks per 64 bytes (quotes, backslashes, control characters, bytes >= 0x80),
// then the odd-length-backslash-run trick to find which quotes are escaped, and a prefix XOR of
// the real quotes to find which bytes are inside strings.
//
// The index only has to be exact for text that stage 1 accepts. If it meets anything it does not
// model (a control character inside a string, invalid UTF-8, a backslash outside a string, an
// unterminated string), build_index returns false and the caller parses without an index, so
// every error comes from the scalar reader and is the same whatever the path.
//
// A simdjson-style index of every structural character was tried and dropped: for documents of
// short strings and numbers it cost more to build than the scalar reader's own byte checks cost
// to repeat (see docs/stdlib/json.md).

#scope_module

CLOSE_FLAG :: 0x4000_0000;
BACKSLASH_FLAG :: 0x8000_0000;
POSITION_MASK :: 0x3fff_ffff;

Block_State :: struct {
    prev_escaped: u64; // the last byte of the previous block was a backslash that escapes the next byte
    prev_in_string: u64; // all ones when the previous block ended inside a string
    utf8_resume: int; // bytes before this offset are the tail of a sequence already validated
}

// The bits of 64 bytes, one bit per byte.
Block_Bits :: struct {
    quote: u64;
    backslash: u64;
    control: u64; // below 0x20
    high: u64; // 0x80 and above
}

classify_chunk :: inline (chunk: u8x16, bits: *Block_Bits, shift: int) {
    bits.quote |= to_bits(eq(chunk, #char "\"")) << shift;
    bits.backslash |= to_bits(eq(chunk, #char "\\")) << shift;
    bits.control |= to_bits(lt(chunk, 0x20)) << shift;
    bits.high |= to_bits(ge(chunk, 0x80)) << shift;
}

// The bit of every byte that follows an odd number of backslashes, in a run that began inside
// this block or carried in. `escaped` bytes are not what they look like: an escaped quote is
// not a quote.
find_escaped :: (state: *Block_State, backslash_bits: u64) -> u64 {
    if backslash_bits == 0 {
        escaped := state.prev_escaped;
        state.prev_escaped = 0;

        return escaped;
    }

    backslash := backslash_bits & ~state.prev_escaped;
    follows_escape := (backslash << 1) | state.prev_escaped;
    even_bits: u64 = 0x5555_5555_5555_5555;
    odd_sequence_starts := backslash & ~even_bits & ~follows_escape;
    sequences_starting_on_even_bits := odd_sequence_starts + backslash;

    state.prev_escaped = ifx sequences_starting_on_even_bits < odd_sequence_starts then cast(u64) 1 else 0;

    invert_mask := sequences_starting_on_even_bits << 1;

    return (even_bits ^ invert_mask) & follows_escape;
}

// Bit i of the result is the XOR of bits 0..i of the input: which bytes lie between an opening
// and a closing quote.
prefix_xor :: (bits: u64) -> u64 {
    x := bits;

    x ^= x << 1;
    x ^= x << 2;
    x ^= x << 4;
    x ^= x << 8;
    x ^= x << 16;
    x ^= x << 32;

    return x;
}

// Adds the entries of one block. Returns false if the block holds something the index does not
// model.
index_block :: (state: *Block_State, block: *u8, base: int, text: string, index: *[..] u32) -> bool {
    bits: Block_Bits;

    classify_chunk(load(u8x16, block), *bits, 0);
    classify_chunk(load(u8x16, block + 16), *bits, 16);
    classify_chunk(load(u8x16, block + 32), *bits, 32);
    classify_chunk(load(u8x16, block + 48), *bits, 48);

    escaped := find_escaped(state, bits.backslash);
    real_quotes := bits.quote & ~escaped;
    in_string := prefix_xor(real_quotes) ^ state.prev_in_string;

    state.prev_in_string = ifx (in_string >> 63) != 0 then 0xffff_ffff_ffff_ffff else cast(u64) 0;

    if (bits.control & in_string) != 0 return false;
    if (bits.backslash & ~in_string) != 0 return false;

    if bits.high != 0 {
        remaining := bits.high;

        while remaining != 0 {
            at := base + trailing_zeros(remaining);

            remaining &= remaining - 1;

            if at < state.utf8_resume continue;

            length := utf8_sequence_length(text, at);

            if length == 0 return false;

            state.utf8_resume = at + length;
        }
    }

    closing := real_quotes & ~in_string;
    opening := real_quotes & in_string;
    inside_backslashes := bits.backslash & in_string;
    all := opening | closing | inside_backslashes;

    if all == 0 return true;

    if index.count + 64 > index.allocated array_reserve(index, max(2 * index.allocated, index.count + 1024));

    count := index.count;
    data := index.data;

    while all != 0 {
        step := trailing_zeros(all);
        bit := (cast(u64) 1) << step;
        entry := cast(u32) (base + step);

        if (closing & bit) != 0 entry |= CLOSE_FLAG;
        else if (inside_backslashes & bit) != 0 entry |= BACKSLASH_FLAG;

        data[count] = entry;
        count += 1;
        all &= all - 1;
    }

    index.count = count;

    return true;
}

// Builds the index of `text` into `index` (cleared first). False if the text is too large for
// 30-bit offsets or stage 1 met something it does not model; `index` is then not usable.
build_index :: (text: string, index: *[..] u32) -> bool {
    index.count = 0;

    if text.count > POSITION_MASK - 64 return false;

    state: Block_State;
    at := 0;

    while at + 64 <= text.count {
        if !index_block(*state, text.data + at, at, text, index) return false;

        at += 64;

        // Strings that are short and numerous, or full of escapes, do not repay the index: the
        // reader scans them as fast as the index can be read. Neither does a text with hardly
        // any strings, such as an array of numbers. Judged on the first 32 KB: it pays between
        // one entry per 48 bytes and one per 2 KB.
        if at == 32768 && (index.count * 48 > at || index.count * 2048 < at) {
            json_index_stats.sparse += 1;

            return false;
        }
    }

    if at < text.count {
        // The last bytes, padded with spaces, which index nothing.
        padded: [64] u8;

        for 0..63 padded[it] = #char " ";

        memcpy(*padded[0], text.data + at, text.count - at);

        if !index_block(*state, *padded[0], at, text, index) return false;
    }

    return state.prev_in_string == 0;
}
