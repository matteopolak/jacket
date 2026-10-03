wildcard_match :: (s: string, pat: string) -> bool {
    cursor: s64;
    pattern: s64;
    star: s64 = -1;
    retry: s64;

    while cursor < s.count {
        if pattern < pat.count && pat[pattern] == 42 {
            star = pattern;
            pattern += 1;
            retry = cursor;
            continue;
        }

        matched, next := match_pattern_byte(pat, pattern, s[cursor]);
        if matched {
            cursor += 1;
            pattern = next;
        } else {
            if star < 0 return false;
            retry += 1;
            cursor = retry;
            pattern = star + 1;
        }
    }

    while pattern < pat.count && pat[pattern] == 42 pattern += 1;

    return pattern == pat.count;
}

#scope_file
match_pattern_byte :: (pat: string, at: s64, byte: u8) -> bool, s64 {
    if at >= pat.count return false, at;
    if pat[at] == 63 return true, at + 1;
    if pat[at] != 91 return pat[at] == byte, at + 1;

    cursor := at + 1;
    matches := false;

    while cursor < pat.count && pat[cursor] != 93 {
        low := pat[cursor];

        if cursor + 2 < pat.count && pat[cursor + 1] == 45 && pat[cursor + 2] != 93 {
            if low <= byte && byte <= pat[cursor + 2] matches = true;
            cursor += 3;
        } else {
            if low == byte matches = true;
            cursor += 1;
        }
    }

    if cursor >= pat.count return false, at;

    return matches, cursor + 1;
}
