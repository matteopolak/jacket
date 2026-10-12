// Unicode lookups for the regex engines: property names to code point ranges, the Perl classes
// and simple case folding. The tables live in unicode_data.jai (generated, see
// docs/tools/regex-unicode-tables.md) as compact text and are decoded one set at a time.

#import "Basic";
#import "Sort";

#load "unicode_data.jai";

#scope_module

// An inclusive span of code points.
Char_Range :: struct {
    lo: u32;
    hi: u32;
}

// Resolves the text of `\p{...}` (without the braces; `\pL` passes "L"). Accepts a bare name
// (general category, script, binary property, `Any`, `ASCII`, `Assigned`) or `Name=Value` /
// `Name:Value` for `gc`, `sc`, `scx` and binary properties (`yes`/`no`). Names match loosely:
// case, spaces, `_` and `-` are ignored, as is a leading `is`. The ranges are sorted and merged.
unicode_property :: (spec: string) -> ranges: [..] Char_Range, ok: bool, error: string {
    none: [..] Char_Range;

    split := -1;
    for spec {
        if it == #char "=" || it == #char ":" {
            split = it_index;
            break;
        }
    }

    if split < 0 {
        found, offset := find_name(.[#char "g", #char "s", #char "b", #char "m"], spec);
        if !found return none, false, sprint("unknown Unicode property `%`", spec);
        return decode_set(offset), true, "";
    }

    name := slice(spec, 0, split);
    value := slice(spec, split + 1, spec.count - split - 1);

    found_kind, kind_letter := find_name(.[#char "P"], name);
    if found_kind {
        found, offset := find_name(.[cast(u8) kind_letter], value);
        if !found return none, false, sprint("unknown value `%` for Unicode property `%`", value, name);
        return decode_set(offset), true, "";
    }

    found, offset := find_name(.[#char "b"], name);
    if !found return none, false, sprint("unknown Unicode property `%`", name);
    wanted := loose_name(value);
    defer array_free(wanted);
    flag := string.{ wanted.count, wanted.data };
    positive := flag == "yes" || flag == "y" || flag == "true" || flag == "t";
    negative := flag == "no" || flag == "n" || flag == "false" || flag == "f";
    if !positive && !negative return none, false, sprint("unknown value `%` for Unicode property `%`", value, name);
    ranges := decode_set(offset);
    if positive return ranges, true, "";
    complemented := ranges_negate(ranges);
    array_free(ranges);
    return complemented, true, "";
}

// `\d`: Nd.
unicode_perl_digit :: () -> [..] Char_Range {
    return decode_set(UNICODE_PERL_DIGIT_OFFSET);
}

// `\s`: White_Space.
unicode_perl_space :: () -> [..] Char_Range {
    return decode_set(UNICODE_PERL_SPACE_OFFSET);
}

// `\w`: Alphabetic, marks, Nd, Pc and Join_Control.
unicode_perl_word :: () -> [..] Char_Range {
    return decode_set(UNICODE_PERL_WORD_OFFSET);
}

// The input plus every simple case-fold orbit member of its code points, merged and sorted.
unicode_case_fold_ranges :: (ranges: [] Char_Range) -> [..] Char_Range {
    load_fold_table();

    sorted: [..] Char_Range;
    array_add(*sorted, ..ranges);
    ranges_normalize(*sorted);

    result: [..] Char_Range;
    array_add(*result, ..sorted);

    next := 0;
    for entry: fold_entries {
        while next < sorted.count && sorted[next].hi < entry.cp next += 1;
        if next == sorted.count break;
        if sorted[next].lo > entry.cp continue;
        for 0..entry.count - 1 {
            other := fold_others[entry.start + it];
            array_add(*result, .{ other, other });
        }
    }

    array_free(sorted);
    ranges_normalize(*result);
    return result;
}

// The other members of `c`'s simple case-fold orbit; empty when it has none. The slice points
// into a table that lives for the whole run: do not modify or free it.
unicode_simple_fold :: (c: u32) -> [] u32 {
    load_fold_table();
    low := 0;
    high := fold_entries.count;
    while low < high {
        middle := (low + high) / 2;
        entry := fold_entries[middle];
        if entry.cp == c return array_view(fold_others, entry.start, entry.count);
        if entry.cp < c low = middle + 1;
        else high = middle;
    }
    return .[];
}

// Sorts and merges overlapping or touching ranges in place.
ranges_normalize :: (ranges: *[..] Char_Range) {
    list := ranges.*;
    if list.count < 2 return;

    sorted := true;
    for 1..list.count - 1 {
        if list[it].lo < list[it - 1].lo {
            sorted = false;
            break;
        }
    }
    if !sorted quick_sort(list, (a: Char_Range, b: Char_Range) -> s64 {
        return cast(s64) a.lo - cast(s64) b.lo;
    });

    used := 0;
    for list {
        if used > 0 && cast(s64) it.lo <= cast(s64) list[used - 1].hi + 1 {
            if it.hi > list[used - 1].hi list[used - 1].hi = it.hi;
        } else {
            list[used] = it;
            used += 1;
        }
    }
    ranges.count = used;
}

// The code points in 0..0x10FFFF that `ranges` does not contain. The input must be normalized.
ranges_negate :: (ranges: [] Char_Range) -> [..] Char_Range {
    result: [..] Char_Range;
    next: s64 = 0;
    for ranges {
        if cast(s64) it.lo > next array_add(*result, .{ cast(u32) next, it.lo - 1 });
        next = cast(s64) it.hi + 1;
    }
    if next <= MAX_CODE_POINT array_add(*result, .{ cast(u32) next, MAX_CODE_POINT });
    return result;
}

#scope_file

MAX_CODE_POINT :: 0x10FFFF;

slice :: (text: string, start: int, count: int) -> string {
    return string.{ count, text.data + start };
}

// ---- reading the encoded tables

// Value of one table digit: 0..31 ends a number, 32..63 means more digits follow.
digit_value :: (c: u8) -> u32 {
    if c >= #char "A" && c <= #char "Z" return c - #char "A";
    if c >= #char "a" && c <= #char "z" return c - #char "a" + 26;
    if c >= #char "0" && c <= #char "9" return c - #char "0" + 52;
    if c == #char "-" return 62;
    return 63;
}

read_number :: (position: *int) -> u32 {
    result: u32 = 0;
    shift: u32 = 0;
    while true {
        c := UNICODE_RANGE_DATA[position.*];
        position.* += 1;
        if c == #char "\n" continue;
        digit := digit_value(c);
        result |= (digit & 31) << shift;
        shift += 5;
        if digit < 32 break;
    }
    return result;
}

decode_set :: (offset: int) -> [..] Char_Range {
    position := offset;
    count := read_number(*position);
    result: [..] Char_Range;
    array_reserve(*result, cast(int) count);
    next: u32 = 0;
    for 1..count {
        lo := next + read_number(*position);
        hi := lo + read_number(*position);
        array_add(*result, .{ lo, hi });
        next = hi + 2;
    }
    return result;
}

// ---- looking names up

// Lower-cases and drops spaces, `_` and `-`.
loose_name :: (name: string) -> [..] u8 {
    result: [..] u8;
    for name {
        c := it;
        if c == #char " " || c == #char "_" || c == #char "-" continue;
        if c >= #char "A" && c <= #char "Z" c += 32;
        array_add(*result, c);
    }
    return result;
}

// Orders `key` against the index line `line`, whose key ends at `=`.
compare_key :: (key: string, line: string) -> int {
    i := 0;
    while true {
        key_done := i >= key.count;
        line_done := i >= line.count || line[i] == #char "=";
        if key_done && line_done return 0;
        if key_done return -1;
        if line_done return 1;
        if key[i] < line[i] return -1;
        if key[i] > line[i] return 1;
        i += 1;
    }
    return 0;
}

// Binary search over the sorted lines of UNICODE_NAME_INDEX for `kind` followed by `name`
// (already normalized). Returns the text after `=`.
index_lookup :: (kind: u8, name: string) -> bool, string {
    key: [..] u8;
    defer array_free(key);
    array_add(*key, kind);
    for name array_add(*key, it);
    needle := string.{ key.count, key.data };

    index := UNICODE_NAME_INDEX;
    low := 0;
    high := index.count;
    while low < high {
        middle := (low + high) / 2;
        start := middle;
        while start > low && index[start - 1] != #char "\n" start -= 1;
        end := middle;
        while end < high && index[end] != #char "\n" end += 1;
        line := slice(index, start, end - start);
        order := compare_key(needle, line);
        if order == 0 {
            equals := 0;
            while line[equals] != #char "=" equals += 1;
            return true, slice(line, equals + 1, line.count - equals - 1);
        }
        if order < 0 high = start;
        else low = end + 1;
    }
    return false, "";
}

parse_decimal :: (text: string) -> int {
    result := 0;
    for text result = result * 10 + (it - #char "0");
    return result;
}

// Tries each kind in order for the loosely matched `name`, with and without a leading `is`.
// For kind `P` the result is the table kind letter as a one-character string.
find_name :: (kinds: [] u8, name: string) -> bool, int {
    normalized := loose_name(name);
    defer array_free(normalized);
    text := string.{ normalized.count, normalized.data };

    for attempt: 0..1 {
        candidate := text;
        if attempt == 1 {
            if text.count <= 2 || text[0] != #char "i" || text[1] != #char "s" break;
            candidate = slice(text, 2, text.count - 2);
        }
        for kind: kinds {
            found, value := index_lookup(kind, candidate);
            if !found continue;
            if kind == #char "P" return true, value[0];
            return true, parse_decimal(value);
        }
    }
    return false, 0;
}

// ---- case folding

Fold_Entry :: struct {
    cp: u32;
    start: int; // first of the other orbit members in fold_others
    count: int;
}

fold_entries: [..] Fold_Entry;
fold_others: [..] u32;
fold_loaded := false;

load_fold_table :: () {
    if fold_loaded return;
    fold_loaded = true;

    position: int = UNICODE_FOLD_OFFSET;
    orbits := read_number(*position);
    members: [..] u32;
    previous: u32 = 0;
    for 1..orbits {
        count := read_number(*position) + 2;
        members.count = 0;
        first := previous + read_number(*position);
        previous = first;
        array_add(*members, first);
        for 2..count array_add(*members, members[members.count - 1] + read_number(*position) + 1);

        for member: members {
            entry: Fold_Entry;
            entry.cp = member;
            entry.start = fold_others.count;
            for other: members {
                if other != member array_add(*fold_others, other);
            }
            entry.count = fold_others.count - entry.start;
            array_add(*fold_entries, entry);
        }
    }
    array_free(members);
    quick_sort(fold_entries, (a: Fold_Entry, b: Fold_Entry) -> s64 {
        return cast(s64) a.cp - cast(s64) b.cp;
    });
}
