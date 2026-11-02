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
    candidates: [..] string;
    directory := path_strip_filename(from);

    if !directory.count directory = ".";

    array_add(*candidates, proto_schema_format("%/%", directory, name));
    for split(s.options.import_paths, ";") if it.count array_add(*candidates, proto_schema_format("%/%", it, name));

    for candidates {
        path, ok := get_absolute_path(it);
        if ok return path;
    }

    return "";
}

proto_load_source :: (s: *proto_Schema, path: string, text: string, token: proto_Token) -> *proto_Source {
    for s.sources if it.path == path {
        if it.active proto_schema_fail(s, token, proto_schema_format("import cycle through '%'", path));
        return it;
    }

    source := New(proto_Source);
    source.path = path;
    source.active = true;
    array_add(*s.sources, source);
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
        builtin := proto_well_known(entry.name);
        child_path := proto_schema_format("<builtin:%>", entry.name);
        child_text := builtin;

        if !builtin.count {
            child_path = proto_import_path(s, path, entry.name);
            ok: bool;
            if child_path.count child_text, ok = read_entire_file(child_path);

            if !ok {
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
