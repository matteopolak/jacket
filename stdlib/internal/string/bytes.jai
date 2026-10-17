DEFAULT_SPACES :: " \t\r\n";
DEFAULT_DELIMITERS :: ";=()";

slice :: inline (s: string, index: s64, count: s64) -> string {
    assert(index >= 0 && count >= 0);
    if index >= s.count return "";
    return .{ min(count, s.count - index), s.data + index };
} @UseResult

compare :: (a: string, b: string) -> int {
    for i: 0..min(a.count, b.count) - 1 {
        if a[i] < b[i] return -1;
        if a[i] > b[i] return 1;
    }

    if a.count < b.count return -1;
    if a.count > b.count return 1;

    return 0;
} @UseResult

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

equal :: (a: string, b: string) -> bool {
    return a == b;
} @UseResult

equal_nocase :: (a: string, b: string) -> bool {
    return compare_nocase(a, b) == 0;
}

begins_with :: (s: string, prefix: string) -> bool {
    return s.count >= prefix.count && slice(s, 0, prefix.count) == prefix;
} @UseResult

ends_with :: (s: string, suffix: string) -> bool {
    return s.count >= suffix.count && slice(s, s.count - suffix.count, suffix.count) == suffix;
} @UseResult

begins_with_nocase :: (s: string, prefix: string) -> bool {
    return s.count >= prefix.count && equal_nocase(slice(s, 0, prefix.count), prefix);
}

ends_with_nocase :: (s: string, suffix: string) -> bool {
    return s.count >= suffix.count && equal_nocase(slice(s, s.count - suffix.count, suffix.count), suffix);
}

is_any :: (c: u8, chars: string) -> bool {
    for byte: chars if byte == c return true;
    return false;
}

find_index_from_left :: (s: string, byte: u8, start_index := 0) -> s64 #no_abc {
    for i: start_index..s.count - 1 if s[i] == byte return i;
    return -1;
} @UseResult

find_index_from_right :: (s: string, byte: u8) -> s64 #no_abc {
    i := s.count;
    while i > 0 {
        i -= 1;
        if s[i] == byte return i;
    }

    return -1;
} @UseResult

find_index_of_any_from_left :: (s: string, bytes: string, start_index := 0) -> s64 #no_abc {
    for i: start_index..s.count - 1 if is_any(s[i], bytes) return i;
    return -1;
}

find_index_of_any_from_right :: (s: string, bytes: string) -> s64 #no_abc {
    i := s.count;
    while i > 0 {
        i -= 1;
        if is_any(s[i], bytes) return i;
    }

    return -1;
}

find_index_from_left :: (s: string, substring: string, start_index := 0) -> s64 {
    if substring.count == 0 return -1;
    for i: start_index..s.count - substring.count if slice(s, i, substring.count) == substring return i;
    return -1;
} @UseResult

find_index_from_right :: (s: string, substring: string) -> s64 {
    if substring.count == 0 return -1;

    i := s.count - substring.count;
    while i >= 0 {
        if slice(s, i, substring.count) == substring return i;
        i -= 1;
    }

    return -1;
} @UseResult

index_of_string_nocase :: (s: string, substring: string) -> int, found: bool {
    if substring.count == 0 return -1, false;
    for i: 0..s.count - substring.count if equal_nocase(slice(s, i, substring.count), substring) return i, true;
    return -1, false;
}

contains :: (s: string, substring: string) -> (found: bool, remainder: string) {
    i := find_index_from_left(s, substring);
    if i < 0 return false, "";
    return true, slice(s, i + substring.count, s.count);
} @UseResult

contains_nocase :: (s: string, substring: string) -> (found: bool, remainder: string) {
    i, found := index_of_string_nocase(s, substring);
    if !found return false, "";
    return true, slice(s, i + substring.count, s.count);
}

contains :: (s: string, char: u8) -> (found: bool, remainder: string) {
    i := find_index_from_left(s, char);
    if i < 0 return false, "";
    return true, slice(s, i + 1, s.count);
} @UseResult

contains_any_character :: (s: string, chars: string) -> (found: bool, remainder: string) {
    i := find_index_of_any_from_left(s, chars);
    if i < 0 return false, "";
    return true, slice(s, i + 1, s.count);
}

contains_any_string :: (s: string, substrings: ..string) -> (found: bool, remainder: string) {
    for i: 0..s.count - 1 for sub : substrings {
        if sub.count > 0 && begins_with(slice(s, i, s.count), sub) return true, slice(s, i + sub.count, s.count);
    }

    return false, "";
}

replace_chars :: (s: string, chars: string, replacement: u8) {
    for i: 0..s.count - 1 if is_any(s[i], chars) s.data[i] = replacement;
}

to_lower_in_place :: ($$s: string) {
    for i: 0..s.count - 1 s.data[i] = to_lower(s[i]);
}

to_upper_in_place :: ($$s: string) {
    for i: 0..s.count - 1 s.data[i] = to_upper(s[i]);
}

to_lower_copy :: (s: string) -> string {
    result := copy_string(s);
    to_lower_in_place(result);
    return result;
} @UseResult

to_upper_copy :: (s: string) -> string {
    result := copy_string(s);
    to_upper_in_place(result);
    return result;
} @UseResult

trim_left :: (s: string, chars := DEFAULT_SPACES) -> string {
    i: s64;
    while i < s.count && is_any(s[i], chars) i += 1;
    return slice(s, i, s.count - i);
} @UseResult

trim_right :: (s: string, chars := DEFAULT_SPACES) -> string {
    count := s.count;
    while count > 0 && is_any(s[count - 1], chars) count -= 1;
    return slice(s, 0, count);
} @UseResult

trim :: (s: string, chars := DEFAULT_SPACES) -> string {
    return trim_right(trim_left(s, chars), chars);
} @UseResult

eat_trailing_spaces :: (_s: string) -> string {
    return trim_right(_s);
}

eat_until_space :: (_s: string) -> string {
    return stop_at_any(_s, DEFAULT_SPACES);
}

stop_at_any :: (s: string, bytes: string) -> string {
    i := find_index_of_any_from_left(s, bytes);
    if i < 0 return s;
    return slice(s, 0, i);
}

normalize_line_endings :: (s: *string) {
    write: s64;

    for read: 0..s.count - 1 {
        if s.data[read] == 13 && read + 1 < s.count && s.data[read + 1] == 10 continue;
        s.data[write] = s.data[read];
        write += 1;
    }

    s.count = write;
}

normalize_line_endings :: (s: string) -> string {
    result := copy_string(s);
    normalize_line_endings(*result);
    return result;
}

split_from_left :: (s: string, byte: u8) -> (found: bool, left: string, right: string) {
    i := find_index_from_left(s, byte);
    if i < 0 return false, s, "";
    return true, slice(s, 0, i), slice(s, i + 1, s.count);
}

split_from_right :: (s: string, byte: u8) -> (found: bool, left: string, right: string) {
    i := find_index_from_right(s, byte);
    if i < 0 return false, "", s;
    return true, slice(s, 0, i), slice(s, i + 1, s.count);
}

split_from_left :: (s: string, separator: string) -> (found: bool, left: string, right: string) {
    i := find_index_from_left(s, separator);
    if i < 0 return false, s, "";
    return true, slice(s, 0, i), slice(s, i + separator.count, s.count);
}

split_from_right :: (s: string, separator: string) -> (found: bool, left: string, right: string) {
    i := find_index_from_right(s, separator);
    if i < 0 return false, s, "";
    return true, slice(s, 0, i), slice(s, i + separator.count, s.count);
}

split_from_left_by_any :: (s: string, bytes: string) -> (found: bool, left: string, right: string) {
    i := find_index_of_any_from_left(s, bytes);
    if i < 0 return false, s, "";
    return true, slice(s, 0, i), slice(s, i + 1, s.count);
}

split_from_right_by_any :: (s: string, bytes: string) -> (found: bool, left: string, right: string) {
    i := find_index_of_any_from_right(s, bytes);
    if i < 0 return false, s, "";
    return true, slice(s, 0, i), slice(s, i + 1, s.count);
}

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
} @UseResult

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
} @UseResult
