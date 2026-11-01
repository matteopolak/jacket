// Which files to search. Folders are searched recursively and in sorted order; folders whose
// name starts with `.` are skipped (one named on the command line is still searched).

// Everything that names a file to search, sorted by path with duplicates removed. A path that
// cannot be read becomes an entry in `errors`. The strings are allocated.
list_files :: (options: Options) -> files: [..] string, errors: [..] string {
    state: Walk_State;
    state.globs = options.globs;

    roots := options.paths;

    if roots.count == 0 roots = string.["."];

    for roots {
        if is_directory(it) {
            if !visit_files(it, true, *state, collect, visit_directories = true, follow_directory_symlinks = false) {
                array_add(*state.errors, heap_print("%: cannot read folder", it));
            }
        } else if file_exists(it) {
            if glob_any(options.globs, it) array_add(*state.files, copy_string(it));
        } else {
            array_add(*state.errors, heap_print("%: no such file or folder", it));
        }
    }

    quick_sort(state.files, compare_strings);
    drop_repeats(*state.files);

    return state.files, state.errors;
}

#scope_file

Walk_State :: struct {
    globs: [] string;
    files: [..] string;
    errors: [..] string;
}

collect :: (info: *File_Visit_Info, state: *Walk_State) {
    if info.had_error {
        array_add(*state.errors, heap_print("%: cannot read", info.full_name));

        return;
    }

    if info.is_directory {
        if begins_with(info.short_name, ".") info.descend_into_directory = false;

        return;
    }

    if glob_any(state.globs, info.full_name) array_add(*state.files, copy_string(info.full_name));
}

// `files` is sorted, so equal names are neighbours.
drop_repeats :: (files: *[..] string) {
    kept := 0;

    for files.* {
        if kept > 0 && files.*[kept - 1] == it {
            free(it);
        } else {
            files.*[kept] = it;
            kept += 1;
        }
    }

    files.count = kept;
}

heap_print :: (format: string, args: ..Any) -> string {
    return copy_string(tprint(format, ..args));
}
