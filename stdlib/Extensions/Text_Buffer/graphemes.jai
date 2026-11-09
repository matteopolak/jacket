// Word movement is an editor policy; Unicode grapheme boundaries are shared with Unicode_Text.

// What a code point counts as when moving by words.
Word_Class :: enum u8 {
    SPACE;
    WORD;
    PUNCTUATION;
}

// The class of a code point for word movement. A run of one class is a word.
word_class :: (c: u32) -> Word_Class {
    if c == #char " " || c == #char "\t" || c == 0xA0 || c == 0x3000 return .SPACE;
    if c >= #char "a" && c <= #char "z" return .WORD;
    if c >= #char "A" && c <= #char "Z" return .WORD;
    if c >= #char "0" && c <= #char "9" return .WORD;
    if c == #char "_" return .WORD;
    if c >= 0x80 && !is_unicode_punctuation(c) return .WORD;

    return .PUNCTUATION;
}

// The offset where the word right of `index` ends. Spaces are skipped first, then a run of word
// characters or of punctuation. Line breaks are the caller's to handle.
//
// ```jai
// assert(next_word("hello world", 0) == 5);
// assert(next_word("hello world", 5) == 11);
// ```
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

// The offset where the word left of `index` starts.
//
// ```jai
// assert(previous_word("hello world", 11) == 6);
// assert(previous_word("hello world", 6) == 0);
// ```
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

// Decodes one scalar, replacing each malformed byte so editing always advances.
// Example: `code, size := decode_at("é", 0);` gives 0xE9 and 2.
decode_at :: (text: string, index: int) -> code: u32, size: int {
    if index < 0 || index >= text.count return 0xFFFD, 0;
    cp, length, status := Unicode.character_utf8_to_utf32(text.data + index, text.count - index, true);
    if status != .CONVERSION_OK return 0xFFFD, 1;
    return cp, length;
}

// The start of the code point that ends at `index`.
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
Unicode :: #import "Unicode";

// Punctuation and symbols outside ASCII that should not glue to words.
is_unicode_punctuation :: (c: u32) -> bool {
    return (c >= 0x2000 && c <= 0x206F) || (c >= 0x3000 && c <= 0x303F) || (c >= 0x00A1 && c <= 0x00BF) || c == 0x00D7 || c == 0x00F7 || (c >= 0x2190 && c <= 0x2BFF) || (c >= 0xFF01 && c <= 0xFF0F);
}
