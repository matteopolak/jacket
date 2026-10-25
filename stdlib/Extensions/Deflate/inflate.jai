#scope_module

// A decode table entry (u32):
//   bits 0..4    bits to consume: code length plus extra bits
//   bits 5..8    code length (for a sub-table pointer: the sub-table's index bits)
//   bits 9..11   kind
//   bits 16..31  literal byte, base length or distance, or a sub-table offset
ENTRY_LITERAL :: 0 << 9;
ENTRY_BASE :: 1 << 9; // length or distance: base value plus extra bits
ENTRY_END :: 2 << 9;
ENTRY_SUB :: 3 << 9;
ENTRY_BAD :: 4 << 9;
ENTRY_KIND :: 7 << 9;

LITERAL_ROOT :: 10;
DISTANCE_ROOT :: 8;
LITERAL_TABLE_SIZE :: 4096;
DISTANCE_TABLE_SIZE :: 1024;

// Output must have this much room for the unchecked fast path: the longest match plus word-copy slack.
FAST_OUTPUT_ROOM :: 274;

make_entry :: inline (kind: u32, value: u32, coded: u32, extra: u32) -> u32 {
    return kind | (value << 16) | (coded + extra) | (coded << 5);
}

// The base value of a length or distance entry plus its extra bits, read from `bits` as they
// were when the entry was looked up.
entry_value :: inline (e: u32, bits: u64) -> s64 {
    one: u64 = 1;
    total: u64 = e & 31;
    code: u64 = (e >> 5) & 15;
    base: s64 = e >> 16;
    extra: u64 = (bits & ((one << total) - 1)) >> code;

    return base + cast(s64) extra;
}

// The entry reached through a sub-table pointer `e`, with `bits` positioned after the root bits.
sub_index :: inline (e: u32, bits: u64) -> u32 {
    one: u64 = 1;
    width: u64 = (e >> 5) & 15;
    index: u64 = bits & ((one << width) - 1);

    return (e >> 16) + cast(u32) index;
}

Inflater :: struct {
    data: *u8;
    size: s64;
    ip: s64;
    bits: u64;
    nbits: s64;

    out: *u8;
    cap: s64;
    o: s64;
    growable: bool;
    max_output: s64;

    fixed_loaded: bool;
    literal_table: [LITERAL_TABLE_SIZE] u32;
    distance_table: [DISTANCE_TABLE_SIZE] u32;
}

// Fills `table` for the canonical code with the given lengths. Rejects over-subscribed codes and
// incomplete ones except a lone code of length 1 (or no code at all when `distances`).
build_table :: (table: *u32, capacity: s64, lengths: *u8, count: s64, root: s64, distances: bool) -> bool {
    counts: [16] s32;
    used: s64;

    for i: 0..count - 1 {
        counts[lengths[i]] += 1;
        if lengths[i] used += 1;
    }

    counts[0] = 0;
    left: s32 = 1;

    for length: 1..15 {
        left = (left << 1) - counts[length];
        if left < 0 return false;
    }

    if left > 0 {
        allowed := used == 1 && counts[1] == 1;
        if distances && used == 0 allowed = true;
        if !allowed return false;
    }

    first: [16] u32;
    code: u32;

    for length: 1..15 {
        code = (code + cast(u32) counts[length - 1]) << 1;
        first[length] = code;
    }

    root_size: u32 = 1 << root;
    // An unused entry consumes the bits it was looked up with, so that a short input can tell
    // a cut-off code from a bad one.
    bad: u32 = ENTRY_BAD | cast(u32) root;

    for i: 0..cast(int) root_size - 1 table[i] = bad;

    // Sub-table index bits per root slot.
    sub_bits: [1 << LITERAL_ROOT] u8;
    reversed: [320] u32;

    for i: 0..count - 1 {
        length := lengths[i];
        if length == 0 continue;

        reversed[i] = reverse_bits(first[length], length);
        first[length] += 1;

        if length > root {
            slot := reversed[i] & (root_size - 1);
            sub_bits[slot] = max(sub_bits[slot], cast(u8) (length - root));
        }
    }

    cursor := root_size;

    for slot: 0..cast(int) root_size - 1 {
        bits := sub_bits[slot];
        if bits == 0 continue;
        if cursor + (1 << bits) > capacity return false;

        sub_entry: u32 = ENTRY_SUB;
        sub_width: u32 = bits;
        sub_offset: u32 = cursor;

        table[slot] = sub_entry | (sub_width << 5) | (sub_offset << 16);

        for j: 0..(1 << bits) - 1 table[cursor + j] = ENTRY_BAD | cast(u32) bits;

        cursor += 1 << bits;
    }

    for i: 0..count - 1 {
        length := cast(s64) lengths[i];
        if length == 0 continue;

        coded := length;
        if length > root coded = length - root;

        coded32 := cast(u32) coded;
        symbol32 := cast(u32) i;
        entry: u32 = ENTRY_BAD | coded32;

        if distances {
            if i < 30 {
                entry = make_entry(ENTRY_BASE, DISTANCE_BASE[i], coded32, DISTANCE_EXTRA[i]);
            }
        } else if i < 256 {
            entry = make_entry(ENTRY_LITERAL, symbol32, coded32, 0);
        } else if i == 256 {
            entry = make_entry(ENTRY_END, 0, coded32, 0);
        } else if i < 286 {
            entry = make_entry(ENTRY_BASE, LENGTH_BASE[i - 257], coded32, LENGTH_EXTRA[i - 257]);
        }

        if length <= root {
            index := reversed[i];
            step: u32 = 1 << length;

            while index < root_size {
                table[index] = entry;
                index += step;
            }
        } else {
            slot := reversed[i] & (root_size - 1);
            offset := table[slot] >> 16;
            size: u32 = 1 << sub_bits[slot];
            index := reversed[i] >> root;
            step: u32 = 1 << (length - root);

            while index < size {
                table[offset + index] = entry;
                index += step;
            }
        }
    }

    return true;
}

load_fixed_tables :: (s: *Inflater) {
    if s.fixed_loaded return;

    lengths: [288] u8;

    for i: 0..287 {
        if i < 144 lengths[i] = 8;
        else if i < 256 lengths[i] = 9;
        else if i < 280 lengths[i] = 7;
        else lengths[i] = 8;
    }

    build_table(s.literal_table.data, LITERAL_TABLE_SIZE, lengths.data, 288, LITERAL_ROOT, false);

    distance_lengths: [32] u8;
    for i: 0..31 distance_lengths[i] = 5;

    build_table(s.distance_table.data, DISTANCE_TABLE_SIZE, distance_lengths.data, 32, DISTANCE_ROOT, true);
    s.fixed_loaded = true;
}

// Tops up the bit buffer one byte at a time; false when the input ended before `count` bits.
need_bits :: inline (s: *Inflater, count: s64) -> bool {
    while s.nbits < count {
        if s.ip >= s.size return false;

        byte: u64 = s.data[s.ip];

        s.bits |= byte << s.nbits;
        s.ip += 1;
        s.nbits += 8;
    }

    return true;
}

take_bits :: inline (s: *Inflater, count: s64) -> u32 {
    one: u64 = 1;
    value := cast(u32) (s.bits & ((one << count) - 1));

    s.bits >>= count;
    s.nbits -= count;

    return value;
}

// Makes room for `needed` bytes of output, or says why not.
grow_output :: (s: *Inflater, needed: s64) -> string {
    if needed <= s.cap return "";
    if !s.growable return "output is too small";
    if needed > s.max_output return "output exceeds the limit";

    new_cap := max(s.cap * 2, needed);
    if new_cap < 4096 new_cap = 4096;
    if new_cap > s.max_output new_cap = s.max_output;

    memory := realloc(s.out, new_cap, s.cap);
    if !memory return "out of memory";

    s.out = memory;
    s.cap = new_cap;

    return "";
}

// Decodes the symbols of one block with the tables in `s`. The fast loop runs while there are
// 8 input bytes to refill from and room for the longest match; everything else is checked.
inflate_codes :: (s: *Inflater) -> string #no_abc {
    data := s.data;
    size := s.size;
    ip := s.ip;
    bits := s.bits;
    nbits := s.nbits;
    out := s.out;
    cap := s.cap;
    o := s.o;
    error := "";
    finished := false;

    literal_root_mask: u64 = (1 << LITERAL_ROOT) - 1;
    distance_root_mask: u64 = (1 << DISTANCE_ROOT) - 1;

    while !finished {
        // Table indices: the primary lookups are masked below the root size; sub-table offsets
        // and widths come from build_table, which keeps them inside the tables.
        while ip + 8 <= size && cap - o >= FAST_OUTPUT_ROOM {
            word := (cast(*u64) (data + ip)).*;

            bits |= word << nbits;
            ip += (63 - nbits) >> 3;
            nbits |= 56;

            e := s.literal_table[bits & literal_root_mask];

            if (e & ENTRY_KIND) == ENTRY_SUB {
                bits >>= LITERAL_ROOT;
                nbits -= LITERAL_ROOT;
                e = s.literal_table[sub_index(e, bits)];
            }

            total: s64 = e & 31;
            kind := e & ENTRY_KIND;

            if kind == ENTRY_LITERAL {
                bits >>= total;
                nbits -= total;
                out[o] = cast,trunc(u8) (e >> 16);
                o += 1;
                continue;
            }

            if kind != ENTRY_BASE {
                if kind == ENTRY_END {
                    bits >>= total;
                    nbits -= total;
                } else {
                    error = "invalid literal/length code";
                }

                finished = true;
                break;
            }

            length := entry_value(e, bits);

            bits >>= total;
            nbits -= total;

            d := s.distance_table[bits & distance_root_mask];

            if (d & ENTRY_KIND) == ENTRY_SUB {
                bits >>= DISTANCE_ROOT;
                nbits -= DISTANCE_ROOT;
                d = s.distance_table[sub_index(d, bits)];
            }

            if (d & ENTRY_KIND) != ENTRY_BASE {
                error = "invalid distance code";
                finished = true;
                break;
            }

            dtotal: s64 = d & 31;
            distance := entry_value(d, bits);

            bits >>= dtotal;
            nbits -= dtotal;

            if distance > o {
                error = "distance is before the start of the output";
                finished = true;
                break;
            }

            copy_match(out + o, distance, length);
            o += length;
        }

        if finished break;

        // One symbol, with every check.
        while nbits <= 56 && ip < size {
            byte: u64 = data[ip];
            bits |= byte << nbits;
            ip += 1;
            nbits += 8;
        }

        e := s.literal_table[bits & literal_root_mask];

        if (e & ENTRY_KIND) == ENTRY_SUB {
            if nbits < LITERAL_ROOT {
                error = "unexpected end of data";
                break;
            }

            bits >>= LITERAL_ROOT;
            nbits -= LITERAL_ROOT;
            e = s.literal_table[sub_index(e, bits)];
        }

        kind := e & ENTRY_KIND;
        total: s64 = e & 31;

        if total > nbits {
            error = "unexpected end of data";
            break;
        }

        if kind == ENTRY_BAD {
            error = "invalid literal/length code";
            break;
        }

        if kind == ENTRY_END {
            bits >>= total;
            nbits -= total;
            break;
        }

        if kind == ENTRY_LITERAL {
            if o >= cap {
                s.out = out;
                error = grow_output(s, o + 1);
                if error.count break;

                out = s.out;
                cap = s.cap;
            }

            bits >>= total;
            nbits -= total;
            out[o] = cast,trunc(u8) (e >> 16);
            o += 1;
            continue;
        }

        length := entry_value(e, bits);

        bits >>= total;
        nbits -= total;

        while nbits <= 56 && ip < size {
            byte: u64 = data[ip];
            bits |= byte << nbits;
            ip += 1;
            nbits += 8;
        }

        d := s.distance_table[bits & distance_root_mask];

        if (d & ENTRY_KIND) == ENTRY_SUB {
            if nbits < DISTANCE_ROOT {
                error = "unexpected end of data";
                break;
            }

            bits >>= DISTANCE_ROOT;
            nbits -= DISTANCE_ROOT;
            d = s.distance_table[sub_index(d, bits)];
        }

        dtotal: s64 = d & 31;

        if dtotal > nbits {
            error = "unexpected end of data";
            break;
        }

        if (d & ENTRY_KIND) != ENTRY_BASE {
            error = "invalid distance code";
            break;
        }

        distance := entry_value(d, bits);

        bits >>= dtotal;
        nbits -= dtotal;

        if distance > o {
            error = "distance is before the start of the output";
            break;
        }

        if o + length > cap {
            s.out = out;
            error = grow_output(s, o + length);
            if error.count break;

            out = s.out;
            cap = s.cap;
        }

        for i: 0..length - 1 out[o + i] = out[o + i - distance];

        o += length;
    }

    s.ip = ip;
    s.bits = bits;
    s.nbits = nbits;
    s.out = out;
    s.cap = cap;
    s.o = o;

    return error;
}

// Copies `length` bytes from `distance` back to `dst`, in whole words that may run up to 7 bytes
// past the end (the caller leaves FAST_OUTPUT_ROOM).
copy_match :: inline (dst_start: *u8, distance: s64, length: s64) #no_abc {
    dst := dst_start;
    end := dst_start + length;

    if distance >= 16 {
        src := dst_start - distance;

        while dst < end {
            first := (cast(*u64) src).*;
            second := (cast(*u64) (src + 8)).*;

            (cast(*u64) dst).* = first;
            (cast(*u64) (dst + 8)).* = second;
            dst += 16;
            src += 16;
        }

        return;
    }

    if distance >= 8 {
        src := dst_start - distance;

        while dst < end {
            (cast(*u64) dst).* = (cast(*u64) src).*;
            dst += 8;
            src += 8;
        }

        return;
    }

    if distance == 1 {
        word := cast(u64) (dst_start - 1).* * 0x0101010101010101;

        while dst < end {
            (cast(*u64) dst).* = word;
            dst += 8;
        }

        return;
    }

    // Short periods: write whole periods until one is at least 8 bytes, then copy words from
    // that period back.
    period := distance;
    while period < 8 period += distance;

    src := dst_start - distance;
    head := min(length, period);

    for i: 0..head - 1 dst_start[i] = src[i]; // jailint: allow(manual_memcpy): the ranges overlap and later bytes repeat earlier ones

    dst = dst_start + head;
    src = dst_start;

    while dst < end {
        (cast(*u64) dst).* = (cast(*u64) src).*;
        dst += 8;
        src += 8;
    }
}
