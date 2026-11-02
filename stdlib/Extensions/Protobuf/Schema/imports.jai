#scope_module

proto_Import :: struct {
    token: proto_Token;
    name: string;
    public: bool;
    source: *proto_Source;
}

proto_Source :: struct {
    path: string;
    active: bool;
    syntax: string;
    features: proto_Features;
    imports: [..] proto_Import;
}

// Canonical disk paths coalesce relative paths, '..' and symlink aliases.
proto_import_path :: (s: *proto_Schema, from: string, name: string) -> string {
    directory := path_strip_filename(from);
    if !directory.count directory = ".";

    // Files in one folder share their answers for the folder-relative candidate.
    candidate := proto_schema_format("%/%", directory, name);
    nearby, fresh := proto_symbols.entry(*s.nearby_paths, candidate);

    if fresh nearby.* = proto_existing_path(candidate);
    if nearby.*.count return nearby.*;

    // The import roots give the same answer for a name wherever it is imported from.
    found, added := proto_symbols.entry(*s.import_roots, name);

    if added {
        for split(s.options.import_paths, ";") if it.count {
            found.* = proto_existing_path(proto_schema_format("%/%", it, name));
            if found.*.count break;
        }
    }

    return found.*;
}

// An absolute path is not proof the file exists: on Windows it never checks.
proto_existing_path :: (candidate: string) -> string {
    path, ok := get_absolute_path(candidate);
    if ok && file_exists(path) return path;
    return "";
}

proto_load_source :: (s: *proto_Schema, path: string, text: string, token: proto_Token) -> *proto_Source {
    known := proto_known_source(s, path, token);
    if known return known;

    source := New(proto_Source);
    source.path = path;
    source.active = true;
    array_add(*s.sources, source);
    proto_symbols.insert(*s.source_paths, path, source);
    local := proto_Schema.{ text = text, options = s.options, source = source };
    local.options.path = path;
    proto_lex(*local);
    if !local.error.message.count proto_parse_schema(*local);

    if local.error.message.count {
        s.error = local.error;
        return source;
    }

    source.imports = local.imports;
    for *entry: source.imports {
        // A file found on disk wins; the built-in well-known copies fill in when there is none.
        child_path := proto_import_path(s, path, entry.name);
        child_text: string;
        ok: bool;

        if child_path.count {
            // A file already loaded is not read again.
            entry.source = proto_known_source(s, child_path, entry.token);
            if s.error.message.count return source;
            if entry.source continue;

            child_text, ok = read_entire_file(child_path);
        }

        if !ok {
            child_text = proto_well_known(entry.name);
            child_path = proto_schema_format("<builtin:%>", entry.name);

            if !child_text.count {
                proto_schema_fail(s, entry.token, proto_schema_format("cannot read import '%'", entry.name));
                return source;
            }
        }

        entry.source = proto_load_source(s, child_path, child_text, entry.token);
        if s.error.message.count return source;
    }

    source.active = false;
    array_add(*s.nodes, ..local.nodes);
    array_add(*s.roots, ..local.roots);

    return source;
}

proto_known_source :: (s: *proto_Schema, path: string, token: proto_Token) -> *proto_Source {
    found := proto_symbols.find(*s.source_paths, path);
    if !found return null;
    if found.*.active proto_schema_fail(s, token, proto_schema_format("import cycle through '%'", path));
    return found.*;
}

proto_load_graph :: (s: *proto_Schema) {
    if s.options.dir.count {
        paths: [..] string;
        ok := visit_files(s.options.dir, true, *paths, (info: *File_Visit_Info, paths: *[..] string) {
            if ends_with(info.short_name, ".proto") array_add(paths, copy_string(info.full_name));
        }, follow_directory_symlinks = false);

        if !ok {
            proto_schema_fail(s, .{ path = s.options.dir, line = 1, column = 1 }, "cannot read schema directory");
            return;
        }

        Sorting.quick_sort(paths, Sorting.compare_strings);

        for paths {
            path, valid := get_absolute_path(it);
            text, read := read_entire_file(path);

            if !valid || !read {
                proto_schema_fail(s, .{ path = it, line = 1, column = 1 }, "cannot read schema");
                return;
            }

            proto_load_source(s, path, text, .{ path = path, line = 1, column = 1 });
            if s.error.message.count return;
        }
    } else {
        path := s.options.path;
        if path != "<text.proto>" {
            absolute, ok := get_absolute_path(path);
            if ok path = absolute;
        }

        proto_load_source(s, path, s.text, .{ path = path, line = 1, column = 1 });
    }
}

// A direct import exposes its own declarations and the transitive public import closure.
proto_public_reaches :: (source: *proto_Source, target: *proto_Source) -> bool {
    if source == target return true;
    for source.imports if it.public && proto_public_reaches(it.source, target) return true;
    return false;
}

proto_visible :: (source: *proto_Source, target: *proto_Source) -> bool {
    if !source || source == target return true;
    for source.imports if proto_public_reaches(it.source, target) return true;
    return false;
}
