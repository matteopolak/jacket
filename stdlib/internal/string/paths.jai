#if OS == .WINDOWS {
    PATH_SEPARATOR :: #char "\\";
} else {
    PATH_SEPARATOR :: #char "/";
}

path_filename :: (path: string) -> string {
    i := find_index_of_any_from_right(path, "/\\");
    return slice(path, i + 1, path.count);
} @UseResult

path_strip_filename :: (path: string) -> string {
    i := find_index_of_any_from_right(path, "/\\");
    return slice(path, 0, i + 1);
}

path_extension :: (path: string) -> (extension: string, found_extension: bool) {
    file := path_filename(path);
    dot := find_index_from_right(file, cast(u8) 46);
    if dot <= 0 return "", false;
    return slice(file, dot + 1, file.count), true;
} @UseResult

path_strip_extension :: (path: string) -> string {
    file := path_filename(path);
    dot := find_index_from_right(file, cast(u8) 46);
    if dot <= 0 return path;
    return slice(path, 0, path.count - file.count + dot);
} @UseResult

path_decomp :: (name: string) -> (path: string, basename: string, extension: string, basename_with_extension: string) {
    directory := path_strip_filename(name);
    file := path_filename(name);
    ext, found := path_extension(file);
    base := file;

    if found base = slice(file, 0, file.count - ext.count - 1);

    return directory, base, ext, file;
}

basename :: (s: string) -> string {
    return path_strip_extension(path_filename(s));
}

path_overwrite_separators :: (path: string, separator: u8 = PATH_SEPARATOR) {
    replace_chars(path, "/\\", separator);
}

path_ensure_extension :: (path: string, extension: string) -> string {
    ext, found := path_extension(path);
    if found && ext == extension return path;
    return join(path, ".", extension);
}

#if OS != .NN_SWITCH2 {
    is_absolute_path :: (path: string) -> bool {
        #if OS == .WINDOWS return path.count >= 3 && is_alpha(path[0]) && path[1] == 58 && is_any(path[2], "/\\");
        else return path.count > 0 && path[0] == 47;
    }
}

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
