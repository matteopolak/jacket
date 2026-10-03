Path :: struct {
    header_string: string;
    number_of_leading_slashes: int;
    trailing_slash: bool;
    words: [..] string;
}

parse_path :: (_s: string, reduce := true) -> Path {
    path: Path;
    rest := _s;

    #if OS == .WINDOWS {
        if rest.count >= 2 && rest[1] == 58 {
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

path_contains :: (path: Path, words: [] string) -> (contains: bool, position: s32) {
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

copy_to_permanent :: (path: Path) -> Path {
    result := path;
    result.header_string = copy_string(path.header_string);
    result.words = .{};
    for word: path.words array_add(*result.words, copy_string(word));

    return result;
}

free_permanent :: (using path: Path) {
    free(header_string);
    for word: words free(word);
    array_free(words);
}

#scope_file
#import "Basic";
S :: #import "String";
