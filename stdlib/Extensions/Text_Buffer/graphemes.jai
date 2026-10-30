// Cursor movement units over UTF-8: user-perceived characters (grapheme clusters) and words.
//
// The cluster rules cover what editors meet in practice: CR LF, combining marks, zero-width joiner
// sequences (family and flag emoji built from several code points), emoji skin tones, variation
// selectors, regional-indicator pairs (flags), tag characters and Hangul jamo. They follow Unicode's
// text segmentation (UAX 29) but are not a full implementation: scripts whose spacing marks join a
// cluster (Indic scripts) are split at those marks.

// The byte offset after the grapheme cluster that starts at `index` in `text`. Returns
// `text.count` at the end, and `index` itself when it is already there.
next_grapheme :: (text: string, index: int) -> int {
    if index >= text.count return text.count;

    first, size := decode_at(text, max(index, 0));
    i := max(index, 0) + size;

    if first == #char "\r" {
        if i < text.count && text.data[i] == #char "\n" return i + 1;
        return i;
    }

    if is_control(first) return i;

    previous := first;
    regional := ifx is_regional_indicator(first) then 1 else 0;

    while i < text.count {
        c, n := decode_at(text, i);

        joins := false;

        if is_extend(c) || c == 0x200D {
            joins = true;
        } else if previous == 0x200D && is_pictographic(c) {
            joins = true;
        } else if is_regional_indicator(c) && regional == 1 {
            joins = true;
            regional = 2;
        } else if is_hangul_pair(previous, c) {
            joins = true;
        }

        if !joins break;

        previous = c;
        i += n;
    }

    return i;
}

// The byte offset where the grapheme cluster that ends at `index` starts. Returns 0 at the start.
previous_grapheme :: (text: string, index: int) -> int {
    end := min(index, text.count);
    if end <= 0 return 0;

    // Clusters are short, so segment forward from a code point boundary a little way back.
    start := max(0, end - 64);
    while start > 0 && (text.data[start] & 0xC0) == 0x80 start -= 1;

    i := start;

    while true {
        next := next_grapheme(text, i);
        if next >= end return i;
        i = next;
    }

    return i;
}

// What a code point counts as when moving by words.
Word_Class :: enum u8 {
    SPACE;
    WORD;
    PUNCTUATION;
}

// The class of a code point for word movement: runs of one class are a word.
word_class :: (c: u32) -> Word_Class {
    if c == #char " " || c == #char "\t" || c == 0xA0 || c == 0x3000 return .SPACE;
    if c >= #char "a" && c <= #char "z" return .WORD;
    if c >= #char "A" && c <= #char "Z" return .WORD;
    if c >= #char "0" && c <= #char "9" return .WORD;
    if c == #char "_" return .WORD;
    if c >= 0x80 && !is_unicode_punctuation(c) return .WORD;

    return .PUNCTUATION;
}

// The byte offset where the word to the right of `index` ends: spaces are skipped, then a run of
// word characters or of punctuation. Stops at the end of the text; line breaks are the caller's.
next_word :: (text: string, index: int) -> int {
    i := clamp(index, 0, text.count);

    while i < text.count {
        c, n := decode_at(text, i);
        if word_class(c) != .SPACE break;
        i += n;
    }

    if i >= text.count return text.count;

    c, n := decode_at(text, i);
    kind := word_class(c);

    while i < text.count {
        c, n = decode_at(text, i);
        if word_class(c) != kind break;
        i += n;
    }

    return i;
}

// The byte offset where the word to the left of `index` starts.
previous_word :: (text: string, index: int) -> int {
    i := clamp(index, 0, text.count);

    while i > 0 {
        p := previous_code_point(text, i);
        c := decode_at(text, p);
        if word_class(c) != .SPACE break;
        i = p;
    }

    if i <= 0 return 0;

    p := previous_code_point(text, i);
    c, n := decode_at(text, p);
    kind := word_class(c);

    while i > 0 {
        p = previous_code_point(text, i);
        c, n = decode_at(text, p);
        if word_class(c) != kind break;
        i = p;
    }

    return i;
}

// Number of grapheme clusters in `text`.
grapheme_count :: (text: string) -> int {
    count := 0;
    i := 0;

    while i < text.count {
        i = next_grapheme(text, i);
        count += 1;
    }

    return count;
}

// Decode the code point at `index`: its value and its length in bytes. Malformed bytes decode as
// U+FFFD, one byte at a time, so scanning always advances.
decode_at :: (text: string, index: int) -> code: u32, size: int {
    b := text.data[index];
    if b < 0x80 return b, 1;

    need := 0;
    code: u32;

    if (b & 0xE0) == 0xC0 {
        need = 1;
        code = b & 0x1F;
    } else if (b & 0xF0) == 0xE0 {
        need = 2;
        code = b & 0x0F;
    } else if (b & 0xF8) == 0xF0 {
        need = 3;
        code = b & 0x07;
    } else {
        return 0xFFFD, 1;
    }

    if index + need >= text.count return 0xFFFD, 1;

    for 1..need {
        cont := text.data[index + it];
        if (cont & 0xC0) != 0x80 return 0xFFFD, 1;
        code = (code << 6) | (cont & 0x3F);
    }

    return code, need + 1;
}

// Start of the code point that ends at `index`.
previous_code_point :: (text: string, index: int) -> int {
    i := min(index, text.count) - 1;
    if i < 0 return 0;

    steps := 0;

    while i > 0 && (text.data[i] & 0xC0) == 0x80 && steps < 3 {
        i -= 1;
        steps += 1;
    }

    // A stray continuation byte with no lead is its own (invalid) code point.
    if (text.data[i] & 0xC0) == 0x80 return min(index, text.count) - 1;

    return i;
}

#scope_file

// Trailing consonants per precomposed Hangul syllable block (including none).
HANGUL_TRAILING_COUNT :: 28;

is_control :: (c: u32) -> bool {
    return c < 0x20 || (c >= 0x7F && c < 0xA0) || c == 0x2028 || c == 0x2029;
}

is_regional_indicator :: (c: u32) -> bool {
    return c >= 0x1F1E6 && c <= 0x1F1FF;
}

is_pictographic :: (c: u32) -> bool {
    return (c >= 0x1F300 && c <= 0x1FAFF) || (c >= 0x2600 && c <= 0x27BF) || (c >= 0x2190 && c <= 0x21FF) || c == 0x00A9 || c == 0x00AE || c == 0x203C || c == 0x2049 || c == 0x2122 || c == 0x2139 || (c >= 0x2300 && c <= 0x23FF) || (c >= 0x2B00 && c <= 0x2BFF) || c == 0x3030 || c == 0x303D || c == 0x3297 || c == 0x3299;
}

// Combining marks, variation selectors, skin tones, tag characters and similar code points that
// never start a cluster.
is_extend :: (c: u32) -> bool {
    if c < 0x300 return false;

    return (c >= 0x0300 && c <= 0x036F)
        || (c >= 0x0483 && c <= 0x0489)
        || (c >= 0x0591 && c <= 0x05BD)
        || (c >= 0x0610 && c <= 0x061A)
        || (c >= 0x064B && c <= 0x065F)
        || c == 0x0670
        || (c >= 0x06D6 && c <= 0x06DC)
        || (c >= 0x0900 && c <= 0x0903)
        || (c >= 0x093A && c <= 0x094F && c != 0x093D)
        || (c >= 0x0E31 && c <= 0x0E3A && c != 0x0E32 && c != 0x0E33)
        || (c >= 0x0E47 && c <= 0x0E4E)
        || (c >= 0x1AB0 && c <= 0x1AFF)
        || (c >= 0x1DC0 && c <= 0x1DFF)
        || c == 0x200C
        || (c >= 0x20D0 && c <= 0x20FF)
        || (c >= 0x302A && c <= 0x302F)
        || (c >= 0x3099 && c <= 0x309A)
        || (c >= 0xFE00 && c <= 0xFE0F)
        || (c >= 0xFE20 && c <= 0xFE2F)
        || (c >= 0x1F3FB && c <= 0x1F3FF)
        || (c >= 0xE0020 && c <= 0xE007F)
        || (c >= 0xE0100 && c <= 0xE01EF);
}

// Hangul: leading + vowel + trailing jamo, and precomposed syllables followed by jamo.
is_hangul_pair :: (a: u32, b: u32) -> bool {
    l := (a >= 0x1100 && a <= 0x115F) || (a >= 0xA960 && a <= 0xA97C);
    v := (a >= 0x1160 && a <= 0x11A7) || (a >= 0xD7B0 && a <= 0xD7C6);
    t := (a >= 0x11A8 && a <= 0x11FF) || (a >= 0xD7CB && a <= 0xD7FB);
    syllable := a >= 0xAC00 && a <= 0xD7A3;
    lv := syllable && ((a - 0xAC00) % HANGUL_TRAILING_COUNT) == 0;

    next_l := (b >= 0x1100 && b <= 0x115F) || (b >= 0xA960 && b <= 0xA97C);
    next_v := (b >= 0x1160 && b <= 0x11A7) || (b >= 0xD7B0 && b <= 0xD7C6);
    next_t := (b >= 0x11A8 && b <= 0x11FF) || (b >= 0xD7CB && b <= 0xD7FB);

    if l && (next_l || next_v) return true;
    if (v || lv) && (next_v || next_t) return true;
    if (t || (syllable && !lv)) && next_t return true;

    return false;
}

// Punctuation and symbols outside ASCII that should not glue to words.
is_unicode_punctuation :: (c: u32) -> bool {
    return (c >= 0x2000 && c <= 0x206F) || (c >= 0x3000 && c <= 0x303F) || (c >= 0x00A1 && c <= 0x00BF) || c == 0x00D7 || c == 0x00F7 || (c >= 0x2190 && c <= 0x2BFF) || (c >= 0xFF01 && c <= 0xFF0F);
}
