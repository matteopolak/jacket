// Which settings govern a file: the nearest jacket.toml above it. The search is here so every
// front end (command line, WASI module, browser playground) runs the same one; each supplies only
// how to read a file, which is why this module needs no file access of its own.

// The settings for files in `directory`: the `[format]` of the nearest jacket.toml in it or one
// of its parents, up to and including `stop` ("" to go to the top), else the defaults. `file` is
// the jacket.toml that was used ("" for the defaults); `config.root` is its directory. When the
// file cannot be parsed, `ok` is false and `error` says why (`line N: ...`).
find_config :: (directory: string, read: Format_File_Reader, stop := "") -> config: Format_Config, file: string, ok: bool, error: string {
    current := directory;

    while true {
        candidate := tprint("%/%", ifx current == "/" then "" else current, JACKET_CONFIG_FILE_NAME);
        text, found := read(candidate);

        if found {
            config, ok, error := parse_config(text);
            config.root = copy_string(current);
            return config, candidate, ok, error;
        }

        if current == stop break;

        parent := parent_directory_of(current);
        if !parent.count || parent == current break;

        current = parent;
    }

    return .{}, "", true, "";
}

// `a/b/c` -> `a/b`; `/a` -> `/`; `a` -> "".
parent_directory_of :: (path: string) -> string {
    end := path.count;
    while end > 1 && path[end - 1] == #char "/" end -= 1;

    slash := find_index_from_right(slice(path, 0, end), #char "/");
    if slash < 0 return "";
    if slash == 0 return "/";

    return slice(path, 0, slash);
}
