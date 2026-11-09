#if OS != .WASM {
    // Pass the asset's folder as user data, pointing to a string. Resolves only contained files.
    // Example: folder := "assets"; resolver := Gltf_Resolver.{gltf_local_resolve, *folder};
    gltf_local_resolve :: (user: *void, uri: string, limit: s64) -> string, bool {
        if !user || limit < 0 || !uri.count return "", false;

        // Decode URI escapes before containment checks, including encoded path separators.
        decoded: String_Builder;
        i: s64;

        while i < uri.count {
            c := uri[i];
            if c == #char "%" {
                if i + 2 >= uri.count {
                    free_buffers(*decoded);
                    return "", false;
                }

                high := hex_digit(uri[i + 1]);
                low := hex_digit(uri[i + 2]);

                if high < 0 || low < 0 {
                    free_buffers(*decoded);
                    return "", false;
                }

                c = cast(u8) (high * 16 + low);
                i += 2;
            }

            if c == 0 || c == #char "\\" || c == #char ":" || c == #char "?" || c == #char "#" {
                free_buffers(*decoded);
                return "", false;
            }

            append(*decoded, string.{ 1, *c });
            i += 1;
        }

        name := builder_to_string(*decoded);

        defer free(name);
        if begins_with(name, "/") return "", false;

        folder := (cast(*string) user).*;
        base, ok := File.get_absolute_path(folder);

        if !ok return "", false;

        defer free(base);
        file: File.File;

        #if OS == .WINDOWS {
            path, valid := File.get_absolute_path(tprint("%/%", base, name));
            if !valid return "", false;

            defer free(path);
            opened: bool;
            file, opened = File.file_open(path);
            if !opened return "", false;

            canonical := File.file_get_canonical_path(file);

            defer free(canonical);

            if !begins_with(canonical, tprint("%/", base)) {
                File.file_close(*file);
                return "", false;
            }
        } else {
            // Anchor each lookup to an open directory; never follow links or parent components.
            cbase := to_c_string(base);

            defer free(cbase);
            descriptor := P.open(cbase, P.O_RDONLY | P.O_DIRECTORY | P.O_NOFOLLOW);
            if descriptor < 0 return "", false;

            start: s64;

            for end: 0..name.count {
                if end < name.count && name[end] != #char "/" continue;

                part := slice(name, start, end - start);

                start = end + 1;
                if part == "." || !part.count continue;

                if part == ".." {
                    _ := P.close(descriptor);
                    return "", false;
                }

                cname := to_c_string(part);
                flags: s32 = P.O_RDONLY | P.O_NOFOLLOW | P.O_NONBLOCK;

                if end < name.count flags |= P.O_DIRECTORY;

                next := P.openat(descriptor, cname, flags);

                free(cname);
                _ := P.close(descriptor);

                if next < 0 return "", false;

                descriptor = next;
            }

            file.handle = P.fdopen(descriptor, "rb");
            if !file.handle {
                _ := P.close(descriptor);
                return "", false;
            }
        }

        defer File.file_close(*file);

        size, sized := File.file_length(file);
        if !sized || size < 0 || size > limit return "", false;

        data := cast(*u8) alloc(size);
        read, count := File.file_read(file, data, size);

        if !read || count != size {
            free(data);
            return "", false;
        }

        return string.{ size, data }, true;
    }
}

#scope_module
#if OS != .WASM {
    File :: #import "File";
    #if OS != .WINDOWS P :: #import "internal/bindings/posix-api";

    hex_digit :: (c: u8) -> s64 {
        if c >= #char "0" && c <= #char "9" return c - #char "0";
        if c >= #char "a" && c <= #char "f" return c - #char "a" + 10;
        if c >= #char "A" && c <= #char "F" return c - #char "A" + 10;
        return -1;
    }
}
