// A path taken apart: its drive, how many slashes open it, whether one closes it, and the names between.
//
// - `header_string` is the drive (`"C:"`) on Windows and empty elsewhere.
// - `number_of_leading_slashes` is `0` for a relative path and `1` or more for a rooted one (`//` is kept as two).
// - `trailing_slash` is whether the text ended in a separator.
// - `words` are the names, with empty ones left out. A word is a slice of the string given to `parse_path`, not a copy, so
//   it is valid only while that string is. The array itself is allocated: free it with `array_free`, or use
//   `copy_to_permanent` for a path that outlives its text and free that with `free_permanent`.
//
// Both `/` and `\` separate names on every target; `path_to_string` writes the platform's own separator.
Path :: struct {
    header_string: string;
    number_of_leading_slashes: int;
    trailing_slash: bool;
    words: [..] string;
}

// Splits the text `_s` into a `Path`, and with `reduce` (the default) tidies it with `reduce_path`.
//
// ```jai
// path := parse_path("..//orchard/./maple/../oak/");
// assert(path.number_of_leading_slashes == 0 && path.trailing_slash);
// assert(path.words.count == 3 && path.words[0] == ".." && path.words[1] == "orchard" && path.words[2] == "oak");
//
// found, position := path_contains(path, .["orchard", "oak"]);
// assert(found && position == 1);
// array_free(path.words);
//
// raw := parse_path("a/./b", reduce = false);
// assert(raw.words.count == 3);
// array_free(raw.words);
// ```
//
// The text is not checked against the file system, and an empty string gives a path with no words. See `Path` for who frees what.
parse_path :: (_s: string, reduce := true) -> Path {
    path: Path;
    rest := _s;

    #if OS == .WINDOWS {
        if rest.count >= 2 && rest[1] == #char ":" {
            path.header_string = S.slice(rest, 0, 2);
            rest = S.slice(rest, 2, rest.count);
        }
    }

    while rest.count > 0 && S.is_any(rest[0], "/\\") {
        path.number_of_leading_slashes += 1;
        advance(*rest);
    }

    path.trailing_slash = _s.count > 0 && S.is_any(_s[_s.count - 1], "/\\");

    while rest.count > 0 {
        found, word, tail := S.split_from_left_by_any(rest, "/\\");
        if word.count > 0 array_add(*path.words, word);
        if !found break;
        rest = tail;
    }

    if reduce {
        result := reduce_path(path);
        array_free(path.words);
        return result;
    }

    return path;
}

// A copy of `path` with the `.` words removed and each `..` cancelled against the word before it.
//
// A `..` with nothing to cancel is kept in a relative path (`../a`), and dropped in a rooted one, where it would go above the
// root. Symbolic links are not looked at, so `a/link/..` becomes `a` even if `link` points elsewhere. The result has a new
// `words` array (free it with `array_free`) whose words are the input's slices; the input is not changed.
reduce_path :: (path: Path) -> Path {
    result := path;
    result.words = .{};

    for word: path.words {
        if word == "." continue;

        if word == ".." {
            if result.words.count > 0 && peek(result.words) != ".." {
                result.words.count -= 1;
                continue;
            }

            if result.number_of_leading_slashes > 0 continue;
        }

        array_add(*result.words, word);
    }

    return result;
}

// Whether the names `words` appear next to each other, in that order, among the words of `path`; and where the first match starts.
//
// The position is `0` when there is no match. An empty `words` matches at position `0`. Names are compared exactly, byte for byte, also on Windows.
path_contains :: (path: Path, words: [] string) -> (contains: bool, position: s32) {
    if words.count > path.words.count return false, 0;

    for i: 0..path.words.count - words.count {
        same := true;

        for word, j: words if path.words[i + j] != word {
            same = false;
            break;
        }

        if same return true, cast(s32) i;
    }

    return false, 0;
}

// Writes `path` back as text in a new string that the caller frees: the drive, the leading slashes, the words with the
// platform separator (`/`, or `\` on Windows) and a closing separator when `trailing_slash` is set and there are words.
//
// `max_words` of `0` or more keeps only that many words, which gives the prefixes of a path one by one; `-1` keeps all.
//
// ```jai
// path := parse_path("/usr/local/bin");
// defer array_free(path.words);
//
// parent := path_to_string(path, 2);
// #if OS == .WINDOWS assert(parent == "\\usr\\local");
// else assert(parent == "/usr/local");
// free(parent);
// ```
path_to_string :: (path: Path, max_words := -1) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, path.header_string);
    for 0..path.number_of_leading_slashes - 1 append(*builder, S.PATH_SEPARATOR);

    count := path.words.count;
    if max_words >= 0 count = min(count, max_words);

    for i: 0..count - 1 {
        if i > 0 append(*builder, S.PATH_SEPARATOR);
        append(*builder, path.words[i]);
    }

    if path.trailing_slash && count > 0 append(*builder, S.PATH_SEPARATOR);

    return builder_to_string(*builder);
}

// A deep copy of `path`: the drive, every word and the array are new allocations from `context.allocator`, so the copy stays valid after the
// text it was parsed from is freed. Give it to `free_permanent`.
copy_to_permanent :: (path: Path) -> Path {
    result := path;
    result.header_string = copy_string(path.header_string);
    result.words = .{};
    for word: path.words array_add(*result.words, copy_string(word));

    return result;
}

// Frees a path made by `copy_to_permanent`: the drive, each word and the array. Do not use it on the result of `parse_path`, whose words are not allocated.
free_permanent :: (using path: Path) {
    free(header_string);
    for word: words free(word);
    array_free(words);
} @FreesArgument

#scope_file
#import "Basic";
S :: #import "String";
