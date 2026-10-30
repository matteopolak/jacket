// The bytes `trim`, `trim_left` and `trim_right` remove by default: space, tab, carriage return and line feed.
DEFAULT_SPACES :: " \t\r\n";
// The bytes `parse_token` treats as tokens of their own by default: `;`, `=`, `(` and `)`.
DEFAULT_DELIMITERS :: ";=()";

// The part of `s` that starts at `index` and holds at most `count` bytes; empty when `index` is
// past the end. Both must not be negative: a negative one fails an assert.
slice :: inline (s: string, index: s64, count: s64) -> string {
    assert(index >= 0 && count >= 0);
    if index >= s.count return "";
    return .{ min(count, s.count - index), s.data + index };
} @UseResult

// Orders two strings byte by byte: -1 when `a` sorts first, 1 when `b` does, 0 when they are equal.
//
// A string that is a prefix of another sorts first. Bytes compare as unsigned numbers, so the order
// is not locale aware and UTF-8 text sorts by code point.
//
// ```jai
// assert(compare("apple", "banana") == -1);
// assert(compare("b", "a") == 1);
// assert(compare("ab", "ab") == 0);
// assert(compare("a", "ab") == -1);
// ```
compare :: (a: string, b: string) -> int {
    count := min(a.count, b.count);

    // Long common prefixes go to memcmp; for short strings the call costs more than the loop.
    if count >= 32 {
        order := memcmp(a.data, b.data, count);
        if order < 0 return -1;
        if order > 0 return 1;
        return ifx a.count < b.count then -1 else ifx a.count > b.count then 1 else 0;
    }

    for i: 0..count - 1 {
        if a[i] < b[i] return -1;
        if a[i] > b[i] return 1;
    }

    if a.count < b.count return -1;
    if a.count > b.count return 1;

    return 0;
} @UseResult

// Like `compare`, but ASCII letters match in either case. Bytes outside `A-Z` and `a-z` compare as they are,
// so this does not fold other scripts.
//
// ```jai
// assert(compare_nocase("Hello", "hELLO") == 0);
// assert(compare_nocase("a", "B") == -1);
// ```
compare_nocase :: (a: string, b: string) -> int {
    for i: 0..min(a.count, b.count) - 1 {
        x := to_lower(a[i]);
        y := to_lower(b[i]);
        if x < y return -1;
        if x > y return 1;
    }

    if a.count < b.count return -1;
    if a.count > b.count return 1;

    return 0;
}

// Whether the two strings hold the same bytes. The same as `a == b`.
equal :: (a: string, b: string) -> bool {
    return a == b;
} @UseResult

// Whether the two strings are equal when ASCII letters are matched in either case.
//
// ```jai
// assert(equal_nocase("MiXeD", "mixed"));
// assert(!equal_nocase("abc", "abcd"));
// ```
equal_nocase :: (a: string, b: string) -> bool {
    return compare_nocase(a, b) == 0;
}

// Whether `s` starts with `prefix`. An empty prefix always matches.
//
// ```jai
// assert(begins_with("filename.txt", "file"));
// assert(!begins_with("file", "filename"));
// ```
begins_with :: (s: string, prefix: string) -> bool {
    return s.count >= prefix.count && slice(s, 0, prefix.count) == prefix;
} @UseResult

// Whether `s` ends with `suffix`. An empty suffix always matches.
//
// ```jai
// assert(ends_with("filename.txt", ".txt"));
// assert(!ends_with("txt", "name.txt"));
// ```
ends_with :: (s: string, suffix: string) -> bool {
    return s.count >= suffix.count && slice(s, s.count - suffix.count, suffix.count) == suffix;
} @UseResult

// Like `begins_with`, with ASCII letters matched in either case.
//
// ```jai
// assert(begins_with_nocase("Content-Length: 5", "content-length"));
// ```
begins_with_nocase :: (s: string, prefix: string) -> bool {
    return s.count >= prefix.count && equal_nocase(slice(s, 0, prefix.count), prefix);
}

// Like `ends_with`, with ASCII letters matched in either case.
//
// ```jai
// assert(ends_with_nocase("PHOTO.JPG", ".jpg"));
// ```
ends_with_nocase :: (s: string, suffix: string) -> bool {
    return s.count >= suffix.count && equal_nocase(slice(s, s.count - suffix.count, suffix.count), suffix);
}

// Whether the byte `c` appears in `chars`.
//
// ```jai
// assert(is_any(#char "b", "abc"));
// assert(!is_any(#char "z", "abc"));
// ```
is_any :: (c: u8, chars: string) -> bool {
    for byte: chars if byte == c return true;
    return false;
}

// Strings of VECTOR_MIN bytes or more are searched with the vector searches, shorter ones one byte
// at a time: a call into the vector search costs more than scanning a word, a line or a path. The
// vector searches are separate procedures so the short scans stay small enough to inline, where a
// constant set like "/\\" is unrolled.
VECTOR_MIN :: 64;

// The index of the first `byte` in `s` at or after `start_index`, or -1 when there is none.
//
// A negative `start_index` starts at the beginning.
//
// ```jai
// assert(find_index_from_left("a/b/c", #char "/") == 1);
// assert(find_index_from_left("a/b/c", #char "/", 2) == 3);
// assert(find_index_from_left("abc", #char "/") == -1);
// ```
find_index_from_left :: (s: string, byte: u8, start_index := 0) -> s64 #no_abc {
    if s.count - start_index >= VECTOR_MIN return vector_find(s, byte, start_index);
    for i: max(start_index, 0)..s.count - 1 if s[i] == byte return i;
    return -1;
} @UseResult

// The index of the last `byte` in `s`, or -1 when there is none.
//
// ```jai
// assert(find_index_from_right("a/b/c", #char "/") == 3);
// ```
find_index_from_right :: (s: string, byte: u8) -> s64 #no_abc {
    if s.count >= VECTOR_MIN return vector_find_last(s, byte);
    return scan_last(s, byte);
} @UseResult

// The last `byte`, one byte at a time. The path helpers use it: a file name or an extension is
// within a few bytes of the end, however long the path.
scan_last :: inline (s: string, byte: u8) -> s64 #no_abc {
    i := s.count;
    while i > 0 {
        i -= 1;
        if s[i] == byte return i;
    }

    return -1;
}

// The index of the first byte of `s` at or after `start_index` that is in `bytes`, or -1 when there is none.
//
// A negative `start_index` starts at the beginning. An empty `bytes` never matches.
//
// ```jai
// assert(find_index_of_any_from_left("key=value;", "=;") == 3);
// assert(find_index_of_any_from_left("key=value;", "=;", 4) == 9);
// ```
find_index_of_any_from_left :: (s: string, bytes: string, start_index := 0) -> s64 #no_abc {
    if s.count - start_index >= VECTOR_MIN return vector_find_any(s, bytes, start_index);
    for i: max(start_index, 0)..s.count - 1 if is_any(s[i], bytes) return i;
    return -1;
}

// The index of the last byte of `s` that is in `bytes`, or -1 when there is none.
//
// ```jai
// assert(find_index_of_any_from_right("dir\\sub/file", "/\\") == 7);
// ```
find_index_of_any_from_right :: (s: string, bytes: string) -> s64 #no_abc {
    if s.count >= VECTOR_MIN return vector_find_last_any(s, bytes);
    return scan_last_any(s, bytes);
}

scan_last_any :: inline (s: string, bytes: string) -> s64 #no_abc {
    i := s.count;
    while i > 0 {
        i -= 1;
        if is_any(s[i], bytes) return i;
    }

    return -1;
}

// Searches for a substring compare at each offset for the first SUBSTRING_PROBE bytes, which is
// where most are found; beyond that, preparing the needle for the vector search pays for itself.
SUBSTRING_PROBE :: 64;
SUBSTRING_VECTOR_MIN :: 256;

// The index where `substring` first appears in `s`, searching from `start_index`, or -1 when it does not.
//
// An empty `substring` is never found, so the result is -1. `start_index` must not be negative, or an
// assert fails (the byte overload accepts a negative one).
//
// ```jai
// assert(find_index_from_left("banana", "ana") == 1);
// assert(find_index_from_left("banana", "ana", 2) == 3);
// assert(find_index_from_left("banana", "") == -1);
// ```
find_index_from_left :: (s: string, substring: string, start_index := 0) -> s64 {
    if substring.count == 0 return -1;

    // Skipping to candidates by their first byte is several times faster than comparing at every offset.
    first := substring.data[0];
    last := s.count - substring.count;

    assert(start_index >= 0, "start_index is negative: %", start_index);
    i := start_index;
    probe_end := min(last, start_index + SUBSTRING_PROBE);

    while i <= probe_end {
        if s.data[i] == first && slice(s, i, substring.count) == substring return i;
        i += 1;
    }

    if i > last return -1;
    if s.count - i >= SUBSTRING_VECTOR_MIN return vector_find_substring(s, substring, i);

    while i <= last {
        if s.data[i] == first && slice(s, i, substring.count) == substring return i;
        i += 1;
    }

    return -1;
} @UseResult

// The index where `substring` last appears in `s`, or -1 when it does not. An empty `substring` is never found.
//
// Matches may overlap: the last `ana` in `banana` starts at 3.
//
// ```jai
// assert(find_index_from_right("banana", "ana") == 3);
// ```
find_index_from_right :: (s: string, substring: string) -> s64 {
    if substring.count == 0 return -1;
    if s.count >= SUBSTRING_VECTOR_MIN return vector_find_last_substring(s, substring);

    i := s.count - substring.count;
    while i >= 0 {
        if slice(s, i, substring.count) == substring return i;
        i -= 1;
    }

    return -1;
} @UseResult

// The index where `substring` first appears in `s` when ASCII letters are matched in either case, and
// whether it was found. When it is not found the index is -1. An empty `substring`, or one longer than `s`,
// is never found.
//
// ```jai
// index, found := index_of_string_nocase("Hello World", "WORLD");
// assert(found && index == 6);
// ```
index_of_string_nocase :: (s: string, substring: string) -> int, found: bool {
    if substring.count == 0 || substring.count > s.count return -1, false;
    for i: 0..s.count - substring.count if equal_nocase(slice(s, i, substring.count), substring) return i, true;
    return -1, false;
}

// Whether `substring` appears in `s`, and the part of `s` after its first match. When it does not appear the
// remainder is empty. An empty `substring` is never found. The remainder points into `s`.
//
// ```jai
// found, rest := contains("copper/blue", "per/");
// assert(found && rest == "blue");
// ```
contains :: (s: string, substring: string) -> (found: bool, remainder: string) {
    i := find_index_from_left(s, substring);
    if i < 0 return false, "";
    return true, slice(s, i + substring.count, s.count);
} @UseResult

// Like `contains`, with ASCII letters matched in either case.
//
// ```jai
// found, rest := contains_nocase("abcDEFghi", "def");
// assert(found && rest == "ghi");
// ```
contains_nocase :: (s: string, substring: string) -> (found: bool, remainder: string) {
    i, found := index_of_string_nocase(s, substring);
    if !found return false, "";
    return true, slice(s, i + substring.count, s.count);
}

// Whether the byte `char` appears in `s`, and the part of `s` after its first occurrence.
//
// ```jai
// found, rest := contains("key=value", #char "=");
// assert(found && rest == "value");
// ```
contains :: (s: string, char: u8) -> (found: bool, remainder: string) {
    i := find_index_from_left(s, char);
    if i < 0 return false, "";
    return true, slice(s, i + 1, s.count);
} @UseResult

// Whether any byte of `chars` appears in `s`, and the part of `s` after the first such byte.
//
// ```jai
// found, rest := contains_any_character("a,b;c", ";,");
// assert(found && rest == "b;c");
// ```
contains_any_character :: (s: string, chars: string) -> (found: bool, remainder: string) {
    i := find_index_of_any_from_left(s, chars);
    if i < 0 return false, "";
    return true, slice(s, i + 1, s.count);
}

// Whether any of the `substrings` appears in `s`, and the part of `s` after the earliest match. At one
// position the first substring listed wins. Empty substrings are ignored.
//
// ```jai
// found, rest := contains_any_string("say hello world", "world", "hello");
// assert(found && rest == " world");
// ```
contains_any_string :: (s: string, substrings: ..string) -> (found: bool, remainder: string) {
    for i: 0..s.count - 1 for sub : substrings {
        if sub.count > 0 && begins_with(slice(s, i, s.count), sub) return true, slice(s, i + sub.count, s.count);
    }

    return false, "";
}

// Overwrites every byte of `s` that is in `chars` with `replacement`. `s` is changed in place, so it
// must not be a string literal.
//
// ```jai
// text := copy_string("a-b_c");
// defer free(text);
// replace_chars(text, "-_", #char " ");
// assert(text == "a b c");
// ```
replace_chars :: (s: string, chars: string, replacement: u8) {
    for i: 0..s.count - 1 if is_any(s[i], chars) s.data[i] = replacement;
}

// Turns the ASCII capitals of `s` into lower case, in place. Other bytes are left as they are, so UTF-8
// text is not folded. `s` must be writable, not a string literal.
//
// ```jai
// text := copy_string("MiXeD");
// defer free(text);
// to_lower_in_place(text);
// assert(text == "mixed");
// ```
to_lower_in_place :: ($$s: string) {
    for i: 0..s.count - 1 s.data[i] = to_lower(s[i]);
}

// Turns the ASCII lower case letters of `s` into capitals, in place. See `to_lower_in_place`.
//
// ```jai
// text := copy_string("MiXeD");
// defer free(text);
// to_upper_in_place(text);
// assert(text == "MIXED");
// ```
to_upper_in_place :: ($$s: string) {
    for i: 0..s.count - 1 s.data[i] = to_upper(s[i]);
}

// A copy of `s` with ASCII capitals turned into lower case. The copy is allocated with the context
// allocator and the caller frees it.
//
// ```jai
// lower := to_lower_copy("Hello");
// defer free(lower);
// assert(lower == "hello");
// ```
to_lower_copy :: (s: string) -> string {
    result := copy_string(s);
    to_lower_in_place(result);
    return result;
} @UseResult @Allocates

// A copy of `s` with ASCII lower case letters turned into capitals. The copy is allocated with the
// context allocator and the caller frees it.
//
// ```jai
// upper := to_upper_copy("Hello");
// defer free(upper);
// assert(upper == "HELLO");
// ```
to_upper_copy :: (s: string) -> string {
    result := copy_string(s);
    to_upper_in_place(result);
    return result;
} @UseResult @Allocates

// `s` without the `chars` at its start; the result points into `s`.
//
// ```jai
// assert(trim_left("  hi ") == "hi ");
// ```
trim_left :: (s: string, chars := DEFAULT_SPACES) -> string {
    i: s64;
    while i < s.count && is_any(s[i], chars) i += 1;
    return slice(s, i, s.count - i);
} @UseResult

// `s` without the `chars` at its end; the result points into `s`.
//
// ```jai
// assert(trim_right("  hi ") == "  hi");
// ```
trim_right :: (s: string, chars := DEFAULT_SPACES) -> string {
    count := s.count;
    while count > 0 && is_any(s[count - 1], chars) count -= 1;
    return slice(s, 0, count);
} @UseResult

// `s` without the `chars` at either end; the result points into `s`.
//
// ```jai
// assert(trim("  hi \t") == "hi");
// assert(trim("--x--", "-") == "x");
// ```
//
// - chars: the bytes to cut off, as a string of them: whitespace by default.
trim :: (s: string, chars := DEFAULT_SPACES) -> string {
    return trim_right(trim_left(s, chars), chars);
} @UseResult

// `_s` without the spaces, tabs and line breaks at its end. The same as `trim_right`.
eat_trailing_spaces :: (_s: string) -> string {
    return trim_right(_s);
}

// The part of `_s` before its first space, tab or line break, or all of it when it has none.
//
// ```jai
// assert(eat_until_space("word and more") == "word");
// assert(eat_until_space("single") == "single");
// ```
eat_until_space :: (_s: string) -> string {
    return stop_at_any(_s, DEFAULT_SPACES);
}

// The part of `s` before the first byte that is in `bytes`, or all of `s` when there is none.
//
// ```jai
// assert(stop_at_any("name=value", "=:") == "name");
// ```
stop_at_any :: (s: string, bytes: string) -> string {
    i := find_index_of_any_from_left(s, bytes);
    if i < 0 return s;
    return slice(s, 0, i);
}

// Removes the carriage return from every `\r\n` pair in `s`, in place, and shortens `s`. A lone `\r` is
// kept. `s` must be writable.
//
// ```jai
// text := copy_string("a\r\nb\rc");
// defer free(text);
// normalize_line_endings(*text);
// assert(text == "a\nb\rc");
// ```
normalize_line_endings :: (s: *string) {
    write: s64;

    for read: 0..s.count - 1 {
        if s.data[read] == #char "\r" && read + 1 < s.count && s.data[read + 1] == #char "\n" continue;
        s.data[write] = s.data[read];
        write += 1;
    }

    s.count = write;
}

// A copy of `s` with the carriage return removed from every `\r\n` pair. A lone `\r` is kept. The copy is
// allocated with the context allocator and the caller frees it.
//
// ```jai
// text := normalize_line_endings("one\r\ntwo");
// defer free(text);
// assert(text == "one\ntwo");
// ```
normalize_line_endings :: (s: string) -> string {
    result := copy_string(s);
    normalize_line_endings(*result);
    return result;
}

// Splits `s` at the first `byte`. Returns whether it was found, the part before it and the part after it.
//
// When the byte is not found the result is `false`, all of `s` and an empty string. Both parts point
// into `s`.
//
// ```jai
// found, left, right := split_from_left("key=a=b", #char "=");
// assert(found && left == "key" && right == "a=b");
//
// found, left, right = split_from_left("none", #char "=");
// assert(!found && left == "none" && right == "");
// ```
split_from_left :: (s: string, byte: u8) -> (found: bool, left: string, right: string) {
    i := find_index_from_left(s, byte);
    if i < 0 return false, s, "";
    return true, slice(s, 0, i), slice(s, i + 1, s.count);
}

// Splits `s` at the last `byte`. Returns whether it was found, the part before it and the part after it.
//
// When the byte is not found the result is `false`, all of `s` and an empty string, as for
// `split_from_left` and the string overload. Both parts point into `s`.
//
// ```jai
// found, left, right := split_from_right("a.b.c", #char ".");
// assert(found && left == "a.b" && right == "c");
//
// found, left, right = split_from_right("abc", #char ".");
// assert(!found && left == "abc" && right == "");
// ```
split_from_right :: (s: string, byte: u8) -> (found: bool, left: string, right: string) {
    i := find_index_from_right(s, byte);
    if i < 0 return false, s, "";
    return true, slice(s, 0, i), slice(s, i + 1, s.count);
}

// Splits `s` at the first `separator`. Returns whether it was found, the part before it and the part after it.
//
// When the separator is not found, or is empty, the result is `false`, all of `s` and an empty string.
//
// ```jai
// found, left, right := split_from_left("a::b::c", "::");
// assert(found && left == "a" && right == "b::c");
// ```
split_from_left :: (s: string, separator: string) -> (found: bool, left: string, right: string) {
    i := find_index_from_left(s, separator);
    if i < 0 return false, s, "";
    return true, slice(s, 0, i), slice(s, i + separator.count, s.count);
}

// Splits `s` at the last `separator`. Returns whether it was found, the part before it and the part after it.
//
// When the separator is not found, or is empty, the result is `false`, all of `s` and an empty string.
//
// ```jai
// found, left, right := split_from_right("a::b::c", "::");
// assert(found && left == "a::b" && right == "c");
// ```
split_from_right :: (s: string, separator: string) -> (found: bool, left: string, right: string) {
    i := find_index_from_right(s, separator);
    if i < 0 return false, s, "";
    return true, slice(s, 0, i), slice(s, i + separator.count, s.count);
}

// Splits `s` at the first byte that is in `bytes`. Returns whether one was found, the part before it and
// the part after it. When none is found the result is `false`, all of `s` and an empty string.
//
// ```jai
// found, left, right := split_from_left_by_any("a,b;c", ",;");
// assert(found && left == "a" && right == "b;c");
// ```
split_from_left_by_any :: (s: string, bytes: string) -> (found: bool, left: string, right: string) {
    i := find_index_of_any_from_left(s, bytes);
    if i < 0 return false, s, "";
    return true, slice(s, 0, i), slice(s, i + 1, s.count);
}

// Splits `s` at the last byte that is in `bytes`. Returns whether one was found, the part before it and
// the part after it. When none is found the result is `false`, all of `s` and an empty string.
//
// ```jai
// found, left, right := split_from_right_by_any("a,b;c", ",;");
// assert(found && left == "a,b" && right == "c");
// ```
split_from_right_by_any :: (s: string, bytes: string) -> (found: bool, left: string, right: string) {
    i := find_index_of_any_from_right(s, bytes);
    if i < 0 return false, s, "";
    return true, slice(s, 0, i), slice(s, i + 1, s.count);
}

// The pieces of `s` between each `separator`, as views into `s`, in a new array that the caller frees. A separator at either end, or two in a row, gives an empty piece.
//
// ```jai
// pieces := split("a,b,,c", #char ",");
// defer array_free(pieces);
// assert(pieces.count == 4 && pieces[2] == "");
// ```
//
// - separator: a string, or a single byte.
split :: (s: string, separator: $T/ .[string, u8]) -> [] string {
    result: [..] string;
    rest := s;

    while true {
        found, left, right := split_from_left(rest, separator);
        array_add(*result, left);
        if !found break;
        rest = right;
    }

    return result;
} @UseResult

// The strings one after another with `separator` between them, in a new string that the caller frees.
//
// ```jai
// joined := join("a", "b", "c", separator = ", ");
// defer free(joined);
// assert(joined == "a, b, c");
// ```
//
// - separator: goes between the strings.
// - before_first: also put the separator before the first string.
// - after_last: also put the separator after the last string.
join :: (inputs: ..string, separator := "", before_first := false, after_last := false) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    if inputs.count == 0 return "";
    if before_first append(*builder, separator);

    for s, i: inputs {
        if i > 0 append(*builder, separator);
        append(*builder, s);
    }

    if after_last append(*builder, separator);

    return builder_to_string(*builder);
} @UseResult @Allocates

// A copy of `s` with every non-overlapping `old` replaced by `new`, and how many were replaced.
//
// An empty `old` replaces nothing. The copy is allocated with the context allocator and the caller
// frees it.
//
// ```jai
// result, count := replace("aaaaa", "aa", "x");
// defer free(result);
// assert(result == "xxa" && count == 2);
// ```
replace :: (s: string, old: string, new: string) -> string, ocurrences: int {
    builder: String_Builder;

    defer free_buffers(*builder);
    count: int;
    rest := s;

    while true {
        found, left, right := split_from_left(rest, old);

        append(*builder, left);
        if !found break;

        append(*builder, new);
        count += 1;
        rest = right;
    }

    return builder_to_string(*builder), count;
} @UseResult @Allocates

#scope_file
Search :: #import "Extensions/Byte_Search";

vector_find :: no_inline (s: string, byte: u8, start: s64) -> s64 {
    return Search.find(s, byte, start);
}

vector_find_last :: no_inline (s: string, byte: u8) -> s64 {
    return Search.find_last(s, byte);
}

// Any number of bytes: up to three have a vector search, more are checked one by one.
vector_find_any :: no_inline (s: string, bytes: string, start: s64) -> s64 {
    if bytes.count == 1 return Search.find(s, bytes[0], start);
    if bytes.count == 2 return Search.find_any(s, bytes[0], bytes[1], start);
    if bytes.count == 3 return Search.find_any(s, bytes[0], bytes[1], bytes[2], start);

    for i: max(start, 0)..s.count - 1 if is_any(s[i], bytes) return i;

    return -1;
}

vector_find_last_any :: no_inline (s: string, bytes: string) -> s64 {
    if bytes.count == 1 return Search.find_last(s, bytes[0]);
    if bytes.count == 2 return Search.find_last_any(s, bytes[0], bytes[1]);
    if bytes.count == 3 return Search.find_last_any(s, bytes[0], bytes[1], bytes[2]);

    i := s.count;
    while i > 0 {
        i -= 1;
        if is_any(s[i], bytes) return i;
    }

    return -1;
}

vector_find_substring :: no_inline (s: string, substring: string, start: s64) -> s64 {
    return Search.find(s, substring, start);
}

vector_find_last_substring :: no_inline (s: string, substring: string) -> s64 {
    return Search.find_last(s, substring);
}
