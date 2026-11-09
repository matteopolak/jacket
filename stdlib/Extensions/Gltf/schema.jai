#scope_module
// Check presence constraints before typed defaults erase absent-versus-empty distinctions.
check_shape :: (value: *Json_Value, path: string, used: *Table(string, bool), options: Gltf_Options, depth: s64 = 0) -> Gltf_Error {
    if depth > options.max_depth return failure(.LIMIT, "schema depth exceeded", path);

    if value.kind == .ARRAY {
        if !value.items.count || value.items.count > options.max_count return failure(.INVALID, "empty or oversized core array", path);

        for *item, i: value.items {
            e := check_shape(item, tprint("%/%", path, i), used, options, depth + 1);
            if e.kind != .NONE return e;
        }
    } else if value.kind == .OBJECT {
        for *m: value.members {
            p := tprint("%/%", path, m.key);

            if m.key == "extras" continue;

            if m.key == "extensions" {
                if m.value.kind != .OBJECT return failure(.INVALID, "extensions must be an object", p);

                for extension: m.value.members {
                    found, _ := table_find(used, extension.key);
                    if !found || extension.value.kind != .OBJECT return failure(.INVALID, "extension must be declared and an object", p);
                }

                continue;
            }

            if m.value.kind == .NULL return failure(.INVALID, "null core property", p);

            e := check_shape(*m.value, p, used, options, depth + 1);
            if e.kind != .NONE return e;
        }

        offset := json_get(value, "byteOffset");
        component := json_get(value, "componentType");
        view := json_get(value, "bufferView");
        count := json_get(value, "count");

        if offset && component && count && !view return failure(.INVALID, "accessor byteOffset requires bufferView", path);
    }

    return .{};
}
