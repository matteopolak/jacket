// Extracts regular files/directories under an existing destination, rejecting all archive links.
// Existing files are never overwritten. On error, earlier outputs remain; discard the destination.
// Example: error := extract(*archive, "unpack", Archive_Limits.{});
extract :: (a: *Archive, dest_dir: string, limits := Archive_Limits.{}) -> Archive_Error {
    if !valid_limits(limits) || a.entries.count > limits.max_entries return problem(.LIMIT, "extraction entry limit");

    names: [..] string;

    defer {
        for names free(it);
        array_free(names);
    }

    seen: Map.Map(string, Archive_Kind);

    defer Map.deinit(*seen);
    total: s64;

    for a.entries {
        if it.size < 0 || it.size > limits.max_entry || it.size > limits.max_total - total return problem(.LIMIT, "extraction size limit", it.offset, it.name);

        total += it.size;
        if it.size > 0 && (it.compressed_size <= 0 || (it.size - 1) / max(it.compressed_size, 1) >= limits.max_ratio) return problem(.LIMIT, "extraction ratio limit", it.offset, it.name);
        if it.kind != .FILE && it.kind != .DIRECTORY return problem(.PATH, "links and special entries are rejected", it.offset, it.name);

        name, valid := safe_name(it.name);
        if !valid return problem(.PATH, "unsafe extraction path", it.offset, it.name);

        array_add(*names, name);
        _, duplicate := Map.insert(*seen, name, it.kind);
        if duplicate return problem(.DUPLICATE, "duplicate extraction path", it.offset, it.name);
    }

    for names {
        for c, i: it {
            if c != #char "/" continue;
            ancestor := slice(it, 0, i);
            found := Map.find(*seen, ancestor);
            if found && found.* != .DIRECTORY return problem(.PATH, "file used as parent directory", 0, it);
        }
    }

    #if OS == .MACOS || OS == .LINUX {
        root := Posix.open(temp_c_string(dest_dir), Posix.O_RDONLY | Posix.O_DIRECTORY | Posix.O_NOFOLLOW | Posix.O_CLOEXEC);
        if root < 0 return problem(.IO, "cannot open extraction directory");

        defer Posix.close(root);

        for e, i: a.entries {
            path := names[i];
            parent := Posix.dup(root);

            if parent < 0 return problem(.IO, "cannot duplicate directory handle", e.offset, e.name);

            defer Posix.close(parent);
            at: s64;
            while at < path.count {
                end := at;
                while end < path.count && path[end] != #char "/" end += 1;

                component := temp_c_string(slice(path, at, end - at));
                directory := end < path.count || e.kind == .DIRECTORY;

                if directory {
                    // Both opens are relative to a pinned directory; no symlink is followed.
                    Posix.mkdirat(parent, component, 493);
                    child := Posix.openat(parent, component, Posix.O_RDONLY | Posix.O_DIRECTORY | Posix.O_NOFOLLOW | Posix.O_CLOEXEC);
                    if child < 0 return problem(.IO, "cannot open safe parent directory", e.offset, e.name);

                    Posix.close(parent);
                    parent = child;
                } else {
                    fd := Posix.openat(parent, component, Posix.O_WRONLY | Posix.O_CREAT | Posix.O_EXCL | Posix.O_NOFOLLOW | Posix.O_CLOEXEC, cast(u32) 420);
                    if fd < 0 return problem(.IO, "cannot create extraction file", e.offset, e.name);

                    error := copy_entry(a, i, Io.Writer.{ data = *fd, write_proc = extraction_write });

                    Posix.close(fd);
                    if error.kind != .NONE return error;
                }

                at = end + 1;
            }
        }

        return .{};
    } else return problem(.UNSUPPORTED, "safe extraction requires macOS or Linux");
}

#scope_module
#if OS == .MACOS || OS == .LINUX {
    Posix :: #import "POSIX";

    extraction_write :: (data: *void, bytes: [] u8) -> s64, Io.Io_Error {
        fd := cast(*s32) data;
        n := Posix.write(fd.*, bytes.data, cast(u64) bytes.count);
        if n <= 0 return 0, .OTHER;
        return n, .NONE;
    }
}

safe_name :: (name: string) -> string, bool {
    if !name.count || name[0] == #char "/" return "", false;

    out: String_Builder;

    defer free_buffers(*out);
    at: s64;
    while at < name.count {
        end := at;
        while end < name.count && name[end] != #char "/" {
            c := name[end];
            if c == 0 || c == #char "\\" || c == #char ":" return "", false;
            end += 1;
        }

        part := slice(name, at, end - at);
        if part == ".." return "", false;

        if part.count && part != "." {
            if builder_string_length(*out) append(*out, "/");
            append(*out, part);
        }

        at = end + 1;
    }

    if !builder_string_length(*out) return "", false;

    return builder_to_string(*out), true;
}
