// Byte_Search: fast searches for bytes and substrings, with vector instructions. A jaic
// extension, not official Jai: other Jai compilers have no such module, and it may change
// between jaic releases. See docs/stdlib/byte-search.md.
//
//     Byte_Search :: #import "Extensions/Byte_Search";
//
//     at := Byte_Search.find(text, #char "\n");          // the first newline, or -1
//     at  = Byte_Search.find_any(text, #char ",", #char ";");
//     at  = Byte_Search.find(text, "needle");              // the first "needle", or -1
//     at  = Byte_Search.find_last(text, #char "/");
//
//     // A needle searched for many times: its analysis is done once.
//     finder := Byte_Search.make_finder("needle");
//     at = Byte_Search.find(*finder, text, start = at + 1);
//
// Every search takes `start` and returns a position in the whole text, or -1. Every search is
// linear in the text, including substring search with a needle built to defeat it.

// The first `byte` at or after `start`, or -1.
find :: (text: string, byte: u8, start := 0) -> int {
    a := byte;
    #insert #run forward_search("eq(chunk, a)", "c == a");
}

// The first of either byte at or after `start`, or -1.
find_any :: (text: string, a: u8, b: u8, start := 0) -> int {
    #insert #run forward_search("eq(chunk, a) | eq(chunk, b)", "c == a || c == b");
}

// The first of any of the three bytes at or after `start`, or -1.
find_any :: (text: string, a: u8, b: u8, c: u8, start := 0) -> int {
    third := c;
    #insert #run forward_search("eq(chunk, a) | eq(chunk, b) | eq(chunk, third)", "c == a || c == b || c == third");
}

// The last `byte` before `end` (-1: the end of the text), or -1.
find_last :: (text: string, byte: u8, end := -1) -> int {
    a := byte;
    #insert #run backward_search("eq(chunk, a)", "c == a");
}

// The last of either byte before `end`, or -1.
find_last_any :: (text: string, a: u8, b: u8, end := -1) -> int {
    #insert #run backward_search("eq(chunk, a) | eq(chunk, b)", "c == a || c == b");
}

// The last of any of the three bytes before `end`, or -1.
find_last_any :: (text: string, a: u8, b: u8, c: u8, end := -1) -> int {
    third := c;
    #insert #run backward_search("eq(chunk, a) | eq(chunk, b) | eq(chunk, third)", "c == a || c == b || c == third");
}

// How many times `byte` occurs in the text.
count :: (text: string, byte: u8) -> int {
    total := 0;
    i := 0;
    if text.count >= LANES {
        a := byte;
        // Matching lanes are 0xFF, which is -1 as a u8: subtracting the mask adds one per match.
        // A u8 lane holds up to 255 of them, so the sums are emptied every 255 chunks.
        while i + LANES <= text.count {
            sums := splat(Bytes, 0);
            rounds := min(255, (text.count - i) / LANES);
            for 1..rounds {
                chunk := load(Bytes, text.data + i);
                sums = sums - bitcast(Bytes, eq(chunk, a));
                i += LANES;
            }
            total += sum_lanes(sums);
        }
    }
    while i < text.count {
        if text[i] == byte total += 1;
        i += 1;
    }
    return total;
}

// The first `needle` at or after `start`, or -1. An empty needle is found at `start`.
find :: (text: string, needle: string, start := 0) -> int {
    if needle.count == 1 return find(text, needle[0], start);
    finder := make_finder(needle);
    return find(*finder, text, start);
}

// The last `needle` ending at or before `end` (-1: the end of the text), or -1.
find_last :: (text: string, needle: string, end := -1) -> int {
    if needle.count == 1 return find_last(text, needle[0], end);
    finder := make_finder(needle);
    return find_last(*finder, text, end);
}

// A needle prepared for searching. It points into the needle, which must outlive it.
Finder :: struct {
    needle: string;
    // The two bytes the vector loop looks for, at these positions in the needle: the rarest two
    // in typical text, so that few candidates need checking.
    first_offset, second_offset: int;
    first_byte, second_byte: u8;
    // Two-Way's factorization, for the linear-time path.
    critical: int;
    period: int;
    periodic: bool;
    // The same for the needle read backwards, for find_last.
    reverse_critical: int;
    reverse_period: int;
    reverse_periodic: bool;
}

make_finder :: (needle: string) -> Finder {
    finder: Finder;
    finder.needle = needle;
    if needle.count == 0 return finder;

    // The rarest byte, then the rarest at a different position.
    first := 0;
    for 1..needle.count - 1 if rarity(needle[it]) < rarity(needle[first]) first = it;
    second := -1;
    for 0..needle.count - 1 {
        if it == first continue;
        if second < 0 || rarity(needle[it]) < rarity(needle[second]) second = it;
    }
    if second < 0 second = first;
    finder.first_offset = min(first, second);
    finder.second_offset = max(first, second);
    finder.first_byte = needle[finder.first_offset];
    finder.second_byte = needle[finder.second_offset];

    finder.critical, finder.period, finder.periodic = factorize(needle, false);
    finder.reverse_critical, finder.reverse_period, finder.reverse_periodic = factorize(needle, true);
    return finder;
}

// The first match at or after `start`, or -1.
find :: (finder: *Finder, text: string, start := 0) -> int {
    needle := finder.needle;
    if start < 0 start = 0;
    if needle.count == 0 return ifx start <= text.count then start else -1;
    if start + needle.count > text.count return -1;
    if needle.count == 1 return find(text, needle[0], start);

    last_start := text.count - needle.count; // the last place a match can begin
    i := start;
    if last_start - i + 1 >= LANES {
        first := splat(Bytes, finder.first_byte);
        second := splat(Bytes, finder.second_byte);
        // Candidates that turned out wrong. While they stay rare compared with the bytes
        // scanned, the vector loop wins; once they don't, the needle and text are the kind that
        // defeat it, and Two-Way takes over to stay linear.
        misses := 0;
        while i + LANES - 1 <= last_start {
            a := load(Bytes, text.data + i + finder.first_offset);
            b := load(Bytes, text.data + i + finder.second_offset);
            bits := to_bits(eq(a, first) & eq(b, second));
            while bits {
                candidate := i + trailing_zeros(bits);
                if memcmp(text.data + candidate, needle.data, needle.count) == 0 return candidate;
                misses += 1;
                bits &= bits - 1;
            }
            i += LANES;
            if misses > 8 && misses * 32 > i - start return two_way_forward(finder, text, i);
        }
    }
    return two_way_forward(finder, text, i);
}

// The last match ending at or before `end` (-1: the end of the text), or -1.
find_last :: (finder: *Finder, text: string, end := -1) -> int {
    needle := finder.needle;
    if end < 0 || end > text.count end = text.count;
    if needle.count == 0 return end;
    if needle.count > end return -1;
    if needle.count == 1 return find_last(text, needle[0], end);

    // i is one past the last place a match may begin.
    i := end - needle.count + 1;
    if i >= LANES {
        first := splat(Bytes, finder.first_byte);
        second := splat(Bytes, finder.second_byte);
        misses := 0;
        while i >= LANES {
            base := i - LANES;
            a := load(Bytes, text.data + base + finder.first_offset);
            b := load(Bytes, text.data + base + finder.second_offset);
            bits := to_bits(eq(a, first) & eq(b, second));
            while bits {
                top := 63 - leading_zeros(bits);
                candidate := base + top;
                if memcmp(text.data + candidate, needle.data, needle.count) == 0 return candidate;
                misses += 1;
                bits &= ~((cast(u64) 1) << top);
            }
            i = base;
            if misses > 8 && misses * 32 > end - i return two_way_backward(finder, text, i + needle.count - 2);
        }
    }
    return two_way_backward(finder, text, i + needle.count - 2);
}

#scope_module

#import "Basic";
#import "Extensions/SIMD";

Bytes :: u8x16;
LANES :: 16;

// The chunked loop shared by the forward byte searches: `test` makes a Mask from `chunk`, and
// `scalar` tests one byte `c`. The last partial chunk is read again overlapping the one before.
forward_search :: (test: string, scalar: string) -> string {
    return tprint(#string JAI
    i := max(start, 0);
    if text.count - i >= LANES {
        // Four chunks at a time, combined, so the loop branches once per 64 bytes.
        while i + 4 * LANES <= text.count {
            p := text.data + i;
            chunk := load(Bytes, p);
            m0 := %1;
            chunk = load(Bytes, p + LANES);
            m1 := %1;
            chunk = load(Bytes, p + 2 * LANES);
            m2 := %1;
            chunk = load(Bytes, p + 3 * LANES);
            m3 := %1;
            if any(m0 | m1 | m2 | m3) {
                if any(m0) return i + first_true(m0);
                if any(m1) return i + LANES + first_true(m1);
                if any(m2) return i + 2 * LANES + first_true(m2);
                return i + 3 * LANES + first_true(m3);
            }
            i += 4 * LANES;
        }
        while i + LANES <= text.count {
            chunk := load(Bytes, text.data + i);
            m := %1;
            if any(m) return i + first_true(m);
            i += LANES;
        }
        if i < text.count {
            last := text.count - LANES;
            chunk := load(Bytes, text.data + last);
            m := %1;
            // Lanes before i were already checked.
            bits := to_bits(m) >> (i - last);
            if bits return i + trailing_zeros(bits);
        }
        return -1;
    }
    while i < text.count {
        c := text[i];
        if %2 return i;
        i += 1;
    }
    return -1;
JAI, test, scalar);
}

// The backward byte searches: the same loop from `end` down.
backward_search :: (test: string, scalar: string) -> string {
    return tprint(#string JAI
    i := ifx end < 0 || end > text.count then text.count else end; // one past the next byte to check
    if i >= LANES {
        while i >= 4 * LANES {
            p := text.data + i - 4 * LANES;
            chunk := load(Bytes, p + 3 * LANES);
            m3 := %1;
            chunk = load(Bytes, p + 2 * LANES);
            m2 := %1;
            chunk = load(Bytes, p + LANES);
            m1 := %1;
            chunk = load(Bytes, p);
            m0 := %1;
            if any(m0 | m1 | m2 | m3) {
                base := i - 4 * LANES;
                if any(m3) return base + 3 * LANES + last_true(m3);
                if any(m2) return base + 2 * LANES + last_true(m2);
                if any(m1) return base + LANES + last_true(m1);
                return base + last_true(m0);
            }
            i -= 4 * LANES;
        }
        while i >= LANES {
            chunk := load(Bytes, text.data + i - LANES);
            m := %1;
            if any(m) return i - LANES + last_true(m);
            i -= LANES;
        }
        if i > 0 {
            chunk := load(Bytes, text.data);
            m := %1;
            // Only lanes below i are left to check.
            bits := to_bits(m) & ((cast(u64) 1 << i) - 1);
            if bits return 63 - leading_zeros(bits);
        }
        return -1;
    }
    while i > 0 {
        i -= 1;
        c := text[i];
        if %2 return i;
    }
    return -1;
JAI, test, scalar);
}

sum_lanes :: (sums: Bytes) -> int {
    total := 0;
    for 0..LANES - 1 total += sums[it];
    return total;
}

// How common a byte is in typical text, data and code: lower is rarer. The vector loop looks for
// the needle's two rarest bytes, so a rough order is all that matters. Letters follow English
// frequencies, then come whitespace, digits, punctuation, and bytes outside ASCII.
rarity :: (byte: u8) -> u8 {
    return RARITY[byte];
}

RARITY :: #run make_rarity();

make_rarity :: () -> [256] u8 {
    table: [256] u8;
    // Control bytes other than whitespace, and bytes 0x80 and up, are rare in text; UTF-8 lead
    // bytes of common scripts less so than the rest.
    for 0..255 table[it] = 10;
    for 0xC2..0xF4 table[it] = 40;
    for 0x80..0xBF table[it] = 60;
    table[0] = 30;
    // Punctuation and symbols.
    for #char "!"..#char "/" table[it] = 50;
    for #char ":"..#char "@" table[it] = 50;
    for #char "["..#char "`" table[it] = 45;
    for #char "{"..#char "~" table[it] = 40;
    table[#char "."] = 120;
    table[#char ","] = 120;
    table[#char "_"] = 90;
    table[#char "-"] = 100;
    table[#char "/"] = 90;
    table[#char "\""] = 90;
    table[#char "("] = 85;
    table[#char ")"] = 85;
    table[#char ";"] = 80;
    table[#char "="] = 80;
    // Digits, 0 and 1 most of all.
    for #char "0"..#char "9" table[it] = 110;
    table[#char "0"] = 140;
    table[#char "1"] = 135;
    // Whitespace.
    table[#char " "] = 255;
    table[#char "\n"] = 150;
    table[#char "\t"] = 110;
    table[#char "\r"] = 70;
    // Letters, most common first; capitals are rarer than their lower case.
    ORDER :: "etaoinshrdlcumwfgypbvkjxqz";
    for ORDER {
        lower := it;
        table[lower] = cast(u8) (250 - it_index * 6);
        table[lower - 32] = cast(u8) (130 - it_index * 3);
    }
    return table;
}

// Two-Way (Crochemore and Perrin): exact search in linear time and constant space. The needle is
// split at a critical position into u and v; v is matched left to right, then u right to left.
// A mismatch in v skips past it; a mismatch in u skips by the period. For a needle that is
// not periodic, skipping by at least half the needle is safe, so there is no memory to keep.

// The critical position and period of the needle, or of the needle read backwards. `periodic`
// says whether u is a suffix of the start of v, which makes the period exact.
factorize :: (needle: string, backwards: bool) -> critical: int, period: int, periodic: bool {
    n := needle.count;
    at :: (i: int) -> u8 #expand {
        return ifx backwards then needle[n - 1 - i] else needle[i];
    }

    // The maximal suffix in one order and in the reverse order; the later one is critical.
    suffix_a, period_a := maximal_suffix(needle, backwards, false);
    suffix_b, period_b := maximal_suffix(needle, backwards, true);
    critical, period := suffix_a, period_a;
    if suffix_b > suffix_a then critical, period = suffix_b, period_b;

    // The needle has period `period` exactly when its first `critical` bytes repeat at
    // `period`; then a mismatch can carry its progress into the next try.
    periodic := critical + period <= n;
    if periodic {
        for 0..critical - 1 if at(it) != at(it + period) {
            periodic = false;
            break;
        }
    }
    if !periodic period = max(critical, n - critical) + 1;
    return critical, period, periodic;
}

// Where the lexicographically greatest suffix begins (or least, with `flip`), and its period.
maximal_suffix :: (needle: string, backwards: bool, flip: bool) -> start: int, period: int {
    n := needle.count;
    at :: (i: int) -> u8 #expand {
        return ifx backwards then needle[n - 1 - i] else needle[i];
    }
    best := 0; // the suffix found so far
    candidate := 1; // the suffix compared against it
    offset := 0; // how far they agree
    period := 1;
    while candidate + offset < n {
        x := at(candidate + offset);
        y := at(best + offset);
        greater := ifx flip then x < y else x > y;
        if x == y {
            if offset + 1 == period {
                candidate += period;
                offset = 0;
            } else {
                offset += 1;
            }
        } else if greater {
            // The candidate is the better suffix.
            best = candidate;
            candidate += 1;
            offset = 0;
            period = 1;
        } else {
            candidate += offset + 1;
            offset = 0;
            period = candidate - best;
        }
    }
    return best, period;
}

two_way_forward :: (finder: *Finder, text: string, start: int) -> int {
    needle := finder.needle;
    n := needle.count;
    critical, period := finder.critical, finder.period;
    pos := start;
    memory := 0; // with a periodic needle: how much of the start is known to match
    while pos + n <= text.count {
        // Match v, from the critical position on.
        i := max(critical, memory);
        while i < n && needle[i] == text[pos + i] i += 1;
        if i < n {
            pos += i - critical + 1;
            memory = 0;
            continue;
        }
        // Then u, backwards.
        j := critical - 1;
        while j >= memory && needle[j] == text[pos + j] j -= 1;
        if j < memory return pos;
        pos += period;
        if finder.periodic memory = n - period;
    }
    return -1;
}

// The mirror of two_way_forward on the needle and text read backwards: matches ending at or
// before `last + 1`, right to left.
two_way_backward :: (finder: *Finder, text: string, last: int) -> int {
    needle := finder.needle;
    n := needle.count;
    critical, period := finder.reverse_critical, finder.reverse_period;
    // `stop` is one past the match's last byte; reading backwards, offset k is needle[n - 1 - k].
    stop := min(last + 1, text.count);
    memory := 0;
    while stop - n >= 0 {
        i := max(critical, memory);
        while i < n && needle[n - 1 - i] == text[stop - 1 - i] i += 1;
        if i < n {
            stop -= i - critical + 1;
            memory = 0;
            continue;
        }
        j := critical - 1;
        while j >= memory && needle[n - 1 - j] == text[stop - 1 - j] j -= 1;
        if j < memory return stop - n;
        stop -= period;
        if finder.reverse_periodic memory = n - period;
    }
    return -1;
}
