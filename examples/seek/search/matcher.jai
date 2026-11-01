// A pattern, ready to look for in lines. A matcher keeps scratch space and a regex search cache,
// so it belongs to one thread: every worker makes its own.

Matcher :: struct {
    fixed: bool;
    ignore_case: bool;
    needle: string; // fixed text; lower case when `ignore_case`
    regex: Regex.Regex;
    folded: [..] u8; // the line being searched, lower-cased (fixed text with `ignore_case`)
}

// Prepares `options.pattern`. `error` is empty when it worked, else says why the pattern is bad.
make_matcher :: (options: Options) -> Matcher, error: string {
    matcher := Matcher.{ fixed = options.fixed, ignore_case = options.ignore_case };

    if options.fixed {
        matcher.needle = ascii_lower(options.pattern, options.ignore_case);

        return matcher, "";
    }

    flags: Regex.Options;
    flags.case_insensitive = options.ignore_case;

    regex, ok, failure := Regex.compile(options.pattern, flags);

    if !ok return .{}, tprint("bad pattern: % (at byte %)", failure.message, failure.offset);

    matcher.regex = regex;

    return matcher, "";
}

free_matcher :: (matcher: *Matcher) {
    free(matcher.needle);
    array_free(matcher.folded);

    if !matcher.fixed && matcher.regex.engine Regex.free_regex(*matcher.regex);

    matcher.* = .{};
}

// The byte offset of the first match in `line`, or -1. A pattern that can match nothing (`x*`)
// matches every line, at 0.
find_in_line :: (matcher: *Matcher, line: string) -> int {
    if !matcher.fixed {
        found, ok := Regex.find(*matcher.regex, line);

        return ifx ok then found.start else -1;
    }

    if matcher.needle.count == 0 return 0;

    if !matcher.ignore_case return find_index_from_left(line, matcher.needle);

    matcher.folded.count = 0;

    for line array_add(*matcher.folded, lower_byte(it));

    return find_index_from_left(string.{ matcher.folded.count, matcher.folded.data }, matcher.needle);
}

#scope_file

lower_byte :: (byte: u8) -> u8 {
    return ifx byte >= #char "A" && byte <= #char "Z" then byte + 32 else byte;
}

// A copy, lower-cased when `lower` says so. Only A-Z change, so offsets in it are offsets in the
// original.
ascii_lower :: (text: string, lower: bool) -> string {
    result := copy_string(text);

    if lower for *result it.* = lower_byte(it.*);

    return result;
}
