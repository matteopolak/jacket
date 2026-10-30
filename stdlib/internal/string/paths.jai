#if OS == .WINDOWS {
    PATH_SEPARATOR :: #char "\\";
} else {
    PATH_SEPARATOR :: #char "/";
}

// The last component of `path`: everything after the last `/` or `\`. Both separators count on every
// platform. The result points into `path`.
//
// ```jai
// assert(path_filename("dir/name.ext") == "name.ext");
// assert(path_filename("dir/") == "");
// ```
path_filename :: (path: string) -> string {
    i := scan_last_any(path, "/\\");
    return slice(path, i + 1, path.count);
} @UseResult

// The directory part of `path`, including its trailing separator. Empty when `path` has no separator.
// Both `/` and `\` count on every platform.
//
// ```jai
// assert(path_strip_filename("dir/name.ext") == "dir/");
// assert(path_strip_filename("name.ext") == "");
// ```
path_strip_filename :: (path: string) -> string {
    i := scan_last_any(path, "/\\");
    return slice(path, 0, i + 1);
}

// The extension of the last component of `path`, without the dot, and whether it has one.
//
// A dot at the start of the name does not start an extension, so `.hidden` has none. With several
// dots the last one counts.
//
// ```jai
// ext, found := path_extension("dir/archive.tar.gz");
// assert(found && ext == "gz");
//
// _, hidden := path_extension("dir/.hidden");
// assert(!hidden);
// ```
path_extension :: (path: string) -> (extension: string, found_extension: bool) {
    file := path_filename(path);
    dot := scan_last(file, cast(u8) 46);
    if dot <= 0 return "", false;
    return slice(file, dot + 1, file.count), true;
} @UseResult

// `path` without the extension of its last component and the dot before it. A path with no extension is
// returned unchanged. The result points into `path`.
//
// ```jai
// assert(path_strip_extension("dir/name.ext") == "dir/name");
// assert(path_strip_extension("dir.d/name") == "dir.d/name");
// ```
path_strip_extension :: (path: string) -> string {
    file := path_filename(path);
    dot := scan_last(file, cast(u8) 46);
    if dot <= 0 return path;
    return slice(path, 0, path.count - file.count + dot);
} @UseResult

// Splits `name` into its directory (with the trailing separator), its name without the extension, the
// extension without the dot, and the name with the extension. All four point into `name`; a missing
// part is empty.
//
// ```jai
// dir, base, ext, file := path_decomp("src/main.jai");
// assert(dir == "src/" && base == "main" && ext == "jai" && file == "main.jai");
// ```
path_decomp :: (name: string) -> (path: string, basename: string, extension: string, basename_with_extension: string) {
    directory := path_strip_filename(name);
    file := path_filename(name);
    ext, found := path_extension(file);
    base := file;

    if found base = slice(file, 0, file.count - ext.count - 1);

    return directory, base, ext, file;
}

// The last component of `s` without its extension.
//
// ```jai
// assert(basename("src/main.jai") == "main");
// ```
basename :: (s: string) -> string {
    return path_strip_extension(path_filename(s));
}

// Replaces every `/` and `\` in `path` with `separator` (the platform's by default), in place. `path` must
// be writable, not a string literal.
//
// ```jai
// path := copy_string("a/b\\c");
// defer free(path);
// path_overwrite_separators(path, #char "/");
// assert(path == "a/b/c");
// ```
path_overwrite_separators :: (path: string, separator: u8 = PATH_SEPARATOR) {
    replace_chars(path, "/\\", separator);
}

// `path` when its extension is already `extension` (without the dot); otherwise a new string with
// `.extension` added. Another extension is kept, so `a.txt` becomes `a.txt.png`.
//
// The result is `path` itself in the first case, which the caller must not free; in the second it is
// allocated with the context allocator and the caller frees it.
//
// ```jai
// same := path_ensure_extension("a.png", "png");
// assert(same == "a.png");
//
// longer := path_ensure_extension("a", "png");
// defer free(longer);
// assert(longer == "a.png");
// ```
path_ensure_extension :: (path: string, extension: string) -> string {
    ext, found := path_extension(path);
    if found && ext == extension return path;
    return join(path, ".", extension);
}

#if OS != .NN_SWITCH2 {
    // Whether `path` is absolute: it starts with `/` elsewhere, or with a drive letter, `:` and a
    // separator on Windows. Not defined on every platform.
    //
    // ```jai
    // #if OS == .WINDOWS assert(is_absolute_path("C:\\dir"));
    // else assert(is_absolute_path("/dir") && !is_absolute_path("dir"));
    // ```
    is_absolute_path :: (path: string) -> bool {
        #if OS == .WINDOWS return path.count >= 3 && is_alpha(path[0]) && path[1] == #char ":" && is_any(path[2], "/\\");
        else return path.count > 0 && path[0] == #char "/";
    }
}

// The components one after another with `separator` between them. A trailing separator on a component
// is dropped first, so no separator is doubled. Empty components still add a separator.
//
// The result is allocated with the context allocator and the caller frees it.
//
// ```jai
// path := path_join("a/", "b", "c.txt", separator = #char "/");
// defer free(path);
// assert(path == "a/b/c.txt");
// ```
//
// - separator: goes between the components; the platform's by default.
path_join :: (components: ..string, separator: u8 = PATH_SEPARATOR) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);

    for part, i: components {
        if i > 0 append(*builder, separator);
        count := part.count;
        if count > 0 && part[count - 1] == separator count -= 1;
        append(*builder, part.data, count);
    }

    return builder_to_string(*builder);
}
