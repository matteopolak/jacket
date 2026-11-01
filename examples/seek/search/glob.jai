// The patterns of `--glob`. `*` matches any run of characters except `/`, `**` any run at all,
// `?` one character, `[abc]` and `[a-z]` one of a set (`[!abc]` anything else). A pattern with no
// `/` is matched against a file's name, one with a `/` against its whole path (a leading `./`
// is ignored).

// Whether `path` is picked by `pattern`.
//
// ```jai
// assert(glob_matches("*.jai", "src/main.jai"));
// assert(!glob_matches("*.jai", "src/main.c"));
// assert(glob_matches("src/*.jai", "./src/main.jai"));
// assert(!glob_matches("src/*.jai", "src/deep/main.jai"));
// assert(glob_matches("src/**.jai", "src/deep/main.jai"));
// ```
glob_matches :: (pattern: string, path: string) -> bool {
    if contains(pattern, "/") {
        candidate := path;

        if begins_with(candidate, "./") candidate = slice(candidate, 2, candidate.count - 2);

        return glob_here(pattern, candidate);
    }

    return glob_here(pattern, path_filename(path));
}

// Whether any of `patterns` picks `path`; an empty list picks everything.
glob_any :: (patterns: [] string, path: string) -> bool {
    if patterns.count == 0 return true;

    for patterns if glob_matches(it, path) return true;

    return false;
}

#scope_file

glob_here :: (pattern: string, text: string) -> bool {
    p := 0;
    t := 0;

    while p < pattern.count {
        c := pattern[p];

        if c == #char "*" {
            crosses := p + 1 < pattern.count && pattern[p + 1] == #char "*";
            rest := slice(pattern, p + ifx crosses then 2 else 1, pattern.count);

            // Try the rest of the pattern at every place the star could end.
            for start: t..text.count {
                if glob_here(rest, slice(text, start, text.count - start)) return true;
                if start < text.count && text[start] == #char "/" && !crosses break;
            }

            return false;
        }

        if t >= text.count return false;

        if c == #char "?" {
            if text[t] == #char "/" return false;

            p += 1;
            t += 1;
        } else if c == #char "[" {
            matched, next := glob_class(pattern, p, text[t]);

            if !matched return false;

            p = next;
            t += 1;
        } else {
            if c != text[t] return false;

            p += 1;
            t += 1;
        }
    }

    return t == text.count;
}

// Reads the `[...]` set starting at `pattern[start]` and tests `byte` against it. Returns the
// index after the set. A set with no closing `]` is a plain `[`.
glob_class :: (pattern: string, start: int, byte: u8) -> bool, int {
    i := start + 1;
    negate := false;

    if i < pattern.count && (pattern[i] == #char "!" || pattern[i] == #char "^") {
        negate = true;
        i += 1;
    }

    first := i;
    found := false;

    while i < pattern.count && (pattern[i] != #char "]" || i == first) {
        low := pattern[i];
        high := low;

        if i + 2 < pattern.count && pattern[i + 1] == #char "-" && pattern[i + 2] != #char "]" {
            high = pattern[i + 2];
            i += 2;
        }

        if byte >= low && byte <= high found = true;

        i += 1;
    }

    if i >= pattern.count return byte == #char "[", start + 1;

    return found != negate, i + 1;
}
