#scope_module

BLOCK_TOKENS :: 1 << 16;
WINDOW :: 32768;
MAX_MATCH :: 258;
SEGMENT :: 1 << 30;

LENGTH_SYMBOL :: #run make_length_symbols();
DISTANCE_SYMBOL_LOW :: #run make_distance_symbols(false);
DISTANCE_SYMBOL_HIGH :: #run make_distance_symbols(true);
TRAILING_ZERO_INDEX :: #run make_trailing_zero_index();

make_length_symbols :: () -> [256] u8 {
    table: [256] u8;

    for symbol: 0..28 {
        count := 1 << LENGTH_EXTRA[symbol];

        for k: 0..count - 1 {
            length := cast(s64) LENGTH_BASE[symbol] + k;
            if length - 3 < 256 table[length - 3] = cast(u8) symbol;
        }
    }

    // Length 258 has its own symbol, which also claims the last slot of symbol 27's range.
    table[255] = 28;

    return table;
}

// Distances up to 256 index by distance - 1, larger ones by (distance - 1) >> 7.
make_distance_symbols :: (high: bool) -> [256] u8 {
    table: [256] u8;

    for symbol: 0..29 {
        count := 1 << DISTANCE_EXTRA[symbol];

        for k: 0..count - 1 {
            distance := cast(s64) DISTANCE_BASE[symbol] + k;
            index := distance - 1;

            if high {
                if index >= 256 table[index >> 7] = cast(u8) symbol;
            } else if index < 256 {
                table[index] = cast(u8) symbol;
            }
        }
    }

    return table;
}

make_trailing_zero_index :: () -> [64] u8 {
    table: [64] u8;

    for i: 0..63 {
        bit: u64 = 1;
        table[((bit << i) * 0x03F79D71B4CB0A89) >> 58] = cast(u8) i;
    }

    return table;
}

distance_symbol :: inline (distance: s64) -> s64 {
    index := distance - 1;

    if index < 256 return DISTANCE_SYMBOL_LOW[index];

    return DISTANCE_SYMBOL_HIGH[index >> 7];
}

Level_Settings :: struct {
    chain: s64; // candidates examined per position
    nice: s64; // stop searching at a match this long
    lazy: s64; // try the next position when the match here is shorter than this (0: greedy)
    good: s64; // after a match this long, search the next position less deeply
    min_match: s64;
}

settings_for :: (level: s64) -> Level_Settings {
    s: Level_Settings;
    s.min_match = 3;

    if level == 1 {
        s.chain = 4;
        s.nice = 16;
        s.min_match = 4;
    } else if level == 2 {
        s.chain = 8;
        s.nice = 24;
        s.min_match = 4;
    } else if level == 3 {
        s.chain = 16;
        s.nice = 32;
        s.min_match = 4;
    } else if level == 4 {
        s.chain = 16;
        s.nice = 32;
        s.lazy = 16;
        s.good = 8;
        s.min_match = 4;
    } else if level == 5 {
        s.chain = 32;
        s.nice = 64;
        s.lazy = 32;
        s.good = 16;
        s.min_match = 4;
    } else if level == 6 {
        s.chain = 20;
        s.nice = 64;
        s.lazy = 24;
        s.good = 12;
    } else if level == 7 {
        s.chain = 64;
        s.nice = 128;
        s.lazy = 64;
        s.good = 32;
    } else if level == 8 {
        s.chain = 256;
        s.nice = 258;
        s.lazy = 200;
        s.good = 64;
    } else {
        s.chain = 1024;
        s.nice = 258;
        s.lazy = 258;
        s.good = 128;
    }

    return s;
}

Encoder :: struct {
    data: *u8;
    size: s64;

    settings: Level_Settings;
    head: *s32;
    prev: *s32;
    hash_shift: u32;
    prev_mask: s64;
    inserted: s64; // positions below this are in the hash chains

    tokens: *u32; // literal: the byte; match: bit 31, (length - 3) << 16, distance - 1
    token_count: s64;
    literal_freq: [288] u32;
    distance_freq: [32] u32;

    out: *u8;
    out_cap: s64;
    out_used: s64;
    acc: u64;
    acc_bits: s64;
}

// Output buffer ---------------------------------------------------------------------------

reserve_output :: (e: *Encoder, extra: s64) {
    if e.out_used + extra <= e.out_cap return;

    new_cap := max(e.out_cap * 2, e.out_used + extra);
    e.out = realloc(e.out, new_cap, e.out_cap);
    e.out_cap = new_cap;
}

// Callers reserve space first; `count` is at most 32.
put_bits :: inline (e: *Encoder, value: u64, count: s64) #no_abc {
    e.acc |= value << cast(u64) e.acc_bits;
    e.acc_bits += count;

    if e.acc_bits >= 32 {
        (cast(*u32) (e.out + e.out_used)).* = cast,trunc(u32) e.acc;
        e.out_used += 4;
        e.acc >>= 32;
        e.acc_bits -= 32;
    }
}

align_output :: (e: *Encoder) {
    while e.acc_bits > 0 {
        e.out[e.out_used] = cast,trunc(u8) e.acc;
        e.out_used += 1;
        e.acc >>= 8;
        e.acc_bits = max(e.acc_bits - 8, 0);
    }

    e.acc = 0;
}

put_byte :: (e: *Encoder, value: u8) {
    reserve_output(e, 1);
    e.out[e.out_used] = value;
    e.out_used += 1;
}

// Match finding ---------------------------------------------------------------------------

hash_at :: inline (e: *Encoder, pos: s64) -> s64 #no_abc {
    word := (cast(*u32) (e.data + pos)).*;

    if e.settings.min_match == 3 word &= 0xFFFFFF;

    return cast(s64) ((word * 0x9E3779B1) >> e.hash_shift);
}

// Number of equal bytes at `a` and `b`, up to `limit`.
match_length :: inline (a: *u8, b: *u8, limit: s64) -> s64 #no_abc {
    i: s64;

    while i + 8 <= limit {
        x := (cast(*u64) (a + i)).* ^ (cast(*u64) (b + i)).*;

        if x != 0 {
            index := ((x & (0 - x)) * 0x03F79D71B4CB0A89) >> 58;
            return i + (cast(s64) TRAILING_ZERO_INDEX[index] >> 3);
        }

        i += 8;
    }

    while i < limit && a[i] == b[i] i += 1;

    return i;
}

// Records `pos` in the chains.
insert_position :: inline (e: *Encoder, pos: s64) #no_abc {
    h := hash_at(e, pos);
    e.prev[pos & e.prev_mask] = e.head[h];
    e.head[h] = cast(s32) pos;
}

// A match longer than this only has its last positions recorded; the rest of a long run is not
// worth the time, since a later match can start from its tail just as well.
MAX_INSERT_RUN :: 64;

// Inserts the positions up to (not including) `end` that the chains do not hold yet, skipping
// the start of a very long stretch.
insert_through :: (e: *Encoder, end: s64) #no_abc {
    last := min(end, e.size - 3);

    if e.inserted < end - MAX_INSERT_RUN e.inserted = end - MAX_INSERT_RUN;

    while e.inserted < last {
        insert_position(e, e.inserted);
        e.inserted += 1;
    }

    if e.inserted < end e.inserted = end;
}

// The longest match at `pos` longer than `beat`, as (length, distance); length 0 if none.
// `pos` has not been inserted: it is, as a side effect, when it has room for a hash.
find_match :: (e: *Encoder, pos: s64, beat: s64, chain_limit: s64) -> s64, s64 #no_abc {
    max_length := min(MAX_MATCH, e.size - pos);

    if pos + 4 > e.size || max_length <= beat {
        if e.inserted == pos e.inserted = pos + 1;
        return 0, 0;
    }

    h := hash_at(e, pos);
    cand := cast(s64) e.head[h];

    e.prev[pos & e.prev_mask] = cast(s32) cand;
    e.head[h] = cast(s32) pos;
    e.inserted = pos + 1;

    here := e.data + pos;
    best := beat;
    best_distance: s64;
    oldest := max(pos - WINDOW - 1, -1); // a distance of exactly WINDOW is allowed
    chain := chain_limit;

    while cand > oldest && chain > 0 {
        there := e.data + cand;

        if there[best] == here[best] {
            length := match_length(here, there, max_length);

            if length > best {
                best = length;
                best_distance = pos - cand;

                if length >= e.settings.nice || length >= max_length break;
            }
        }

        next := cast(s64) e.prev[cand & e.prev_mask];
        if next >= cand break;

        cand = next;
        chain -= 1;
    }

    if best_distance == 0 return 0, 0;

    // A 3-byte match from far away costs more than three literals.
    if best == 3 && best_distance > 4096 return 0, 0;

    return best, best_distance;
}

// Block output --------------------------------------------------------------------------

add_literal :: inline (e: *Encoder, byte: u8) #no_abc {
    e.tokens[e.token_count] = byte;
    e.token_count += 1;
    e.literal_freq[byte] += 1;
}

add_match :: inline (e: *Encoder, length: s64, distance: s64) #no_abc {
    length_field := cast(u32) (length - 3);
    distance_field := cast(u32) (distance - 1);

    e.tokens[e.token_count] = 0x80000000 | (length_field << 16) | distance_field;
    e.token_count += 1;
    e.literal_freq[257 + cast(s64) LENGTH_SYMBOL[length - 3]] += 1;
    e.distance_freq[distance_symbol(distance)] += 1;
}

FIXED_LITERAL_LENGTHS :: #run make_fixed_literal_lengths();

make_fixed_literal_lengths :: () -> [288] u8 {
    lengths: [288] u8;

    for i: 0..287 {
        if i < 144 lengths[i] = 8;
        else if i < 256 lengths[i] = 9;
        else if i < 280 lengths[i] = 7;
        else lengths[i] = 8;
    }

    return lengths;
}

Code_Set :: struct {
    literal_lengths: [288] u8;
    literal_codes: [288] u16;
    distance_lengths: [32] u8;
    distance_codes: [32] u16;
}

// Writes the tokens with the given codes, then the end-of-block code.
write_tokens :: (e: *Encoder, codes: *Code_Set) #no_abc {
    // A token takes at most 48 bits, and the caller reserved the whole block.
    for i: 0..e.token_count - 1 {
        token := e.tokens[i];

        if token < 0x80000000 {
            put_bits(e, codes.literal_codes[token], codes.literal_lengths[token]);
            continue;
        }

        length := cast(s64) ((token >> 16) & 0xFF);
        distance := cast(s64) (token & 0xFFFF) + 1;
        symbol := cast(s64) LENGTH_SYMBOL[length];
        extra := cast(s64) LENGTH_EXTRA[symbol];
        coded := 257 + symbol;
        low := cast(u64) (length + 3 - cast(s64) LENGTH_BASE[symbol]);

        literal_code: u64 = codes.literal_codes[coded];
        literal_length: s64 = codes.literal_lengths[coded];

        put_bits(e, literal_code | (low << literal_length), literal_length + extra);

        dsymbol := distance_symbol(distance);
        dextra := cast(s64) DISTANCE_EXTRA[dsymbol];
        dlow := cast(u64) (distance - cast(s64) DISTANCE_BASE[dsymbol]);

        distance_code: u64 = codes.distance_codes[dsymbol];
        distance_length: s64 = codes.distance_lengths[dsymbol];

        put_bits(e, distance_code | (dlow << distance_length), distance_length + dextra);
    }

    put_bits(e, codes.literal_codes[256], codes.literal_lengths[256]);
}

// Bits needed for the token counts under the given code lengths, extra bits included.
tokens_cost :: (e: *Encoder, literal_lengths: *u8, distance_lengths: *u8) -> s64 {
    cost: s64;

    for i: 0..285 cost += cast(s64) e.literal_freq[i] * cast(s64) literal_lengths[i];
    for i: 0..29 cost += cast(s64) e.distance_freq[i] * cast(s64) distance_lengths[i];

    for i: 0..28 cost += cast(s64) e.literal_freq[257 + i] * cast(s64) LENGTH_EXTRA[i];
    for i: 0..29 cost += cast(s64) e.distance_freq[i] * cast(s64) DISTANCE_EXTRA[i];

    return cost;
}

// Gives unused symbols a frequency until two are in use, so every alphabet has a complete code.
ensure_two :: (freq: *u32, count: s64) {
    used: s64;

    for i: 0..count - 1 if freq[i] used += 1;

    i: s64;

    while used < 2 {
        if freq[i] == 0 {
            freq[i] = 1;
            used += 1;
        }

        i += 1;
    }
}

Header_Plan :: struct {
    literal_count: s64;
    distance_count: s64;
    code_length_count: s64;
    symbols: [320] u8; // run-length symbols of the code lengths
    extras: [320] u8;
    symbol_count: s64;
    code_lengths: [19] u8;
    code_length_codes: [19] u16;
    bits: s64; // size of the header after the 3 block bits
}

// Run-length codes the literal and distance lengths and sizes the header.
plan_header :: (plan: *Header_Plan, codes: *Code_Set) {
    plan.literal_count = 286;
    while plan.literal_count > 257 && codes.literal_lengths[plan.literal_count - 1] == 0 plan.literal_count -= 1;

    plan.distance_count = 30;
    while plan.distance_count > 1 && codes.distance_lengths[plan.distance_count - 1] == 0 plan.distance_count -= 1;

    total := plan.literal_count + plan.distance_count;
    all: [320] u8;

    memcpy(all.data, codes.literal_lengths.data, plan.literal_count * size_of(u8));
    for i: 0..plan.distance_count - 1 all[plan.literal_count + i] = codes.distance_lengths[i];

    freq: [19] u32;
    i: s64;
    n: s64;

    while i < total {
        value := all[i];
        run: s64 = 1;

        while i + run < total && all[i + run] == value run += 1;

        i += run;

        if value == 0 {
            while run >= 11 {
                take := min(run, 138);

                plan.symbols[n] = 18;
                plan.extras[n] = cast(u8) (take - 11);
                n += 1;
                run -= take;
            }

            if run >= 3 {
                plan.symbols[n] = 17;
                plan.extras[n] = cast(u8) (run - 3);
                n += 1;
                run = 0;
            }
        } else {
            plan.symbols[n] = value;
            plan.extras[n] = 0;
            n += 1;
            run -= 1;

            while run >= 3 {
                take := min(run, 6);

                plan.symbols[n] = 16;
                plan.extras[n] = cast(u8) (take - 3);
                n += 1;
                run -= take;
            }
        }

        // What is left is short: single copies.
        while run > 0 {
            plan.symbols[n] = value;
            plan.extras[n] = 0;
            n += 1;
            run -= 1;
        }
    }

    plan.symbol_count = n;

    for k: 0..n - 1 freq[plan.symbols[k]] += 1;

    ensure_two(freq.data, 19);
    build_code_lengths(freq.data, 19, 7, plan.code_lengths.data);
    assign_codes(plan.code_lengths.data, 19, plan.code_length_codes.data);

    plan.code_length_count = 19;
    while plan.code_length_count > 4 && plan.code_lengths[CODE_LENGTH_ORDER[plan.code_length_count - 1]] == 0 plan.code_length_count -= 1;

    bits := 14 + 3 * plan.code_length_count;

    for k: 0..n - 1 {
        symbol := plan.symbols[k];

        bits += cast(s64) plan.code_lengths[symbol];

        if symbol == 16 bits += 2;
        else if symbol == 17 bits += 3;
        else if symbol == 18 bits += 7;
    }

    plan.bits = bits;
}

write_header :: (e: *Encoder, plan: *Header_Plan) {
    put_bits(e, cast(u64) (plan.literal_count - 257), 5);
    put_bits(e, cast(u64) (plan.distance_count - 1), 5);
    put_bits(e, cast(u64) (plan.code_length_count - 4), 4);

    for i: 0..plan.code_length_count - 1 put_bits(e, plan.code_lengths[CODE_LENGTH_ORDER[i]], 3);

    for k: 0..plan.symbol_count - 1 {
        symbol := plan.symbols[k];

        put_bits(e, plan.code_length_codes[symbol], cast(s64) plan.code_lengths[symbol]);

        if symbol == 16 put_bits(e, plan.extras[k], 2);
        else if symbol == 17 put_bits(e, plan.extras[k], 3);
        else if symbol == 18 put_bits(e, plan.extras[k], 7);
    }
}

write_stored :: (e: *Encoder, start: s64, end: s64, final: bool) {
    at := start;

    while true {
        length := min(end - at, 65535);
        last := at + length >= end;

        reserve_output(e, length + 16);
        put_bits(e, ifx final && last then 1 else 0, 1);
        put_bits(e, 0, 2);
        align_output(e);

        e.out[e.out_used] = cast,trunc(u8) length;
        e.out[e.out_used + 1] = cast,trunc(u8) (length >> 8);
        e.out[e.out_used + 2] = cast,trunc(u8) ~length;
        e.out[e.out_used + 3] = cast,trunc(u8) (~length >> 8);
        e.out_used += 4;

        if length memcpy(e.out + e.out_used, e.data + at, length);

        e.out_used += length;
        at += length;

        if last break;
    }
}

// Ends the current block, whose input bytes are [start, end), with the cheapest encoding.
flush_block :: (e: *Encoder, start: s64, end: s64, final: bool) {
    input_bytes := end - start;
    stored_bits := (input_bytes + 5 * (input_bytes / 65535 + 1)) * 8;

    e.literal_freq[256] = 1;

    codes: Code_Set;
    plan: Header_Plan;

    // The frequencies are only counted for the cost estimate; dummies for completeness go on a copy.
    literal_freq := e.literal_freq;
    distance_freq := e.distance_freq;

    ensure_two(literal_freq.data, 286);
    ensure_two(distance_freq.data, 30);

    build_code_lengths(literal_freq.data, 286, 15, codes.literal_lengths.data);
    build_code_lengths(distance_freq.data, 30, 15, codes.distance_lengths.data);

    plan_header(*plan, *codes);

    dynamic_bits := plan.bits + tokens_cost(e, codes.literal_lengths.data, codes.distance_lengths.data);

    fixed_distance: [32] u8;

    for i: 0..31 fixed_distance[i] = 5;

    fixed_bits := tokens_cost(e, FIXED_LITERAL_LENGTHS.data, fixed_distance.data);

    reserve_output(e, input_bytes + 5 * (input_bytes / 65535 + 1) + 16);

    if stored_bits <= fixed_bits && stored_bits <= dynamic_bits {
        write_stored(e, start, end, final);
    } else if fixed_bits <= dynamic_bits {
        fixed: Code_Set;

        memcpy(fixed.literal_lengths.data, FIXED_LITERAL_LENGTHS.data, 288 * size_of(u8));
        for i: 0..31 fixed.distance_lengths[i] = 5;

        assign_codes(fixed.literal_lengths.data, 288, fixed.literal_codes.data);
        assign_codes(fixed.distance_lengths.data, 32, fixed.distance_codes.data);

        put_bits(e, ifx final then 3 else 2, 3);
        write_tokens(e, *fixed);
    } else {
        assign_codes(codes.literal_lengths.data, 286, codes.literal_codes.data);
        assign_codes(codes.distance_lengths.data, 30, codes.distance_codes.data);

        put_bits(e, ifx final then 5 else 4, 3);
        write_header(e, *plan);
        write_tokens(e, *codes);
    }

    e.token_count = 0;
    for i: 0..287 e.literal_freq[i] = 0;
    for i: 0..31 e.distance_freq[i] = 0;
}

// The main loop --------------------------------------------------------------------------

compress_blocks :: (e: *Encoder, last_segment: bool) {
    n := e.size;
    s := e.settings;
    pos: s64;
    block_start: s64;
    pending_length: s64; // a match found at `pos` while looking ahead from the position before
    pending_distance: s64;

    token_limit := min(BLOCK_TOKENS, n + 1);

    while pos < n {
        length := pending_length;
        distance := pending_distance;

        pending_length = 0;

        if length == 0 length, distance = find_match(e, pos, s.min_match - 1, s.chain);

        if length == 0 {
            add_literal(e, e.data[pos]);
            pos += 1;
        } else {
            if s.lazy && length < s.lazy && pos + 1 < n {
                chain := s.chain;
                if length >= s.good chain >>= 2;

                next_length, next_distance := find_match(e, pos + 1, length, chain);

                if next_length > length {
                    add_literal(e, e.data[pos]);
                    pos += 1;
                    pending_length = next_length;
                    pending_distance = next_distance;

                    if e.token_count >= token_limit {
                        flush_block(e, block_start, pos, false);
                        block_start = pos;
                    }

                    continue;
                }
            }

            add_match(e, length, distance);
            insert_through(e, pos + length);
            pos += length;
        }

        if e.token_count >= token_limit {
            flush_block(e, block_start, pos, false);
            block_start = pos;
        }
    }

    flush_block(e, block_start, n, last_segment);
}

compress :: (data: [] u8, container: Deflate_Container, level: s32) -> [] u8 {
    e: Encoder;
    e.data = data.data;
    e.size = data.count;
    e.out_cap = 64 + data.count / 2;
    e.out = alloc(e.out_cap);

    lvl := cast(s64) clamp(level, 0, 9);

    if container == .ZLIB {
        flevel := 2;
        if lvl == 1 flevel = 0;
        else if lvl < 6 flevel = 1;
        else if lvl > 6 flevel = 3;

        cmf := 0x78;
        flg := flevel << 6;
        flg += (31 - (cmf * 256 + flg) % 31) % 31;

        put_byte(*e, cast(u8) cmf);
        put_byte(*e, cast(u8) flg);
    } else if container == .GZIP {
        header := u8.[0x1F, 0x8B, 8, 0, 0, 0, 0, 0, 0, 255];
        header[8] = ifx lvl == 9 then 2 else ifx lvl == 1 then 4 else 0;

        for header put_byte(*e, it);
    }

    if data.count == 0 {
        if lvl == 0 {
            write_stored(*e, 0, 0, true);
        } else {
            reserve_output(*e, 16);
            put_bits(*e, 3, 3);
            put_bits(*e, 0, 7);
        }
    } else if lvl == 0 {
        write_stored(*e, 0, data.count, true);
    } else {
        e.settings = settings_for(lvl);

        hash_bits: u32 = 8;
        while hash_bits < 15 && (cast(s64) 1 << hash_bits) < min(data.count, SEGMENT) hash_bits += 1;

        e.hash_shift = 32 - hash_bits;

        one: s64 = 1;
        head_count := one << hash_bits;

        e.head = alloc(head_count * size_of(s32));
        for i: 0..head_count - 1 e.head[i] = -1;

        prev_count := min(cast(s64) WINDOW, head_count * 2);
        if prev_count < 256 prev_count = 256;

        prev_count = min(prev_count, WINDOW);
        while prev_count < WINDOW && prev_count < data.count prev_count *= 2;

        e.prev_mask = prev_count - 1;
        e.prev = alloc(prev_count * size_of(s32));
        e.tokens = alloc(min(BLOCK_TOKENS, data.count + 1) * size_of(u32));

        // Positions in the chains are 32-bit, so a huge input goes in independent segments.
        start: s64;

        while true {
            e.data = data.data + start;
            e.size = min(SEGMENT, data.count - start);
            e.inserted = 0;

            if start > 0 for i: 0..head_count - 1 e.head[i] = -1;

            last := start + e.size >= data.count;

            compress_blocks(*e, last);

            start += e.size;
            if last break;
        }

        free(e.head);
        free(e.prev);
        free(e.tokens);
    }

    reserve_output(*e, 16);
    align_output(*e);

    if container == .ZLIB {
        adler := adler32_update(data, 1);

        put_byte(*e, cast,trunc(u8) (adler >> 24));
        put_byte(*e, cast,trunc(u8) (adler >> 16));
        put_byte(*e, cast,trunc(u8) (adler >> 8));
        put_byte(*e, cast,trunc(u8) adler);
    } else if container == .GZIP {
        crc := crc32_update(data, 0);
        size := cast(u32) (data.count & 0xFFFFFFFF);

        for 0..3 put_byte(*e, cast,trunc(u8) (crc >> cast(u32) (it * 8)));
        for 0..3 put_byte(*e, cast,trunc(u8) (size >> cast(u32) (it * 8)));
    }

    result: [] u8;
    result.data = e.out;
    result.count = e.out_used;

    return result;
}
