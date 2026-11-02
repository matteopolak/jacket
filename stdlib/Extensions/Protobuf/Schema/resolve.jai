#scope_module

proto_SCALARS :: string.["int32", "int64", "uint32", "uint64", "sint32", "sint64", "fixed32", "fixed64", "sfixed32", "sfixed64", "float", "double", "bool", "string", "bytes"];
proto_JAI_TYPES :: string.["s32", "s64", "u32", "u64", "s32", "s64", "u32", "u64", "s32", "s64", "float32", "float64", "bool", "string", "[] u8"];
proto_KEYWORDS :: string.["if", "else", "for", "while", "loop", "break", "continue", "return", "defer", "using", "struct", "union", "enum", "enum_flags", "flags", "type", "Type", "true", "false", "null", "cast", "autocast", "xx", "new", "delete", "remove", "then", "case", "inline", "no_inline", "bake", "sizeof", "size_of", "type_of", "type_info", "align_of", "void", "bool", "int", "s8", "s16", "s32", "s64", "u8", "u16", "u32", "u64", "float", "float32", "float64", "string", "Any", "Context", "Code", "it", "it_index", "operator", "push_context", "push_allocator", "context", "reschedule", "yield", "assert", "debug", "print", "log"];

// Buckets index the fixed keyword list. Exact comparison handles hash collisions.
proto_KEYWORD_BUCKETS :: #run -> [128] s64 {
    result: [128] s64;
    for *result it.* = -1;

    for proto_KEYWORDS {
        bucket := (it.count * 17 + cast(s64) it[0] * 3 + it[it.count - 1]) % result.count;
        result[bucket] = it_index;
    }

    return result;
};

proto_KEYWORD_PREVIOUS :: #run -> [proto_KEYWORDS.count] s64 {
    heads: [128] s64;
    for *heads it.* = -1;

    result: [proto_KEYWORDS.count] s64;

    for proto_KEYWORDS {
        bucket := (it.count * 17 + cast(s64) it[0] * 3 + it[it.count - 1]) % heads.count;
        result[it_index] = heads[bucket];
        heads[bucket] = it_index;
    }

    return result;
};

proto_jai_name :: (name: string) -> string {
    if name == "_unknown" || begins_with(name, "proto_") return proto_schema_format("%_", name);

    if name.count {
        bucket := (name.count * 17 + cast(s64) name[0] * 3 + name[name.count - 1]) % proto_KEYWORD_BUCKETS.count;
        index := proto_KEYWORD_BUCKETS[bucket];

        while index >= 0 {
            if name == proto_KEYWORDS[index] return proto_schema_format("%_", name);
            index = proto_KEYWORD_PREVIOUS[index];
        }
    }

    return name;
}

proto_qualified_jai :: (name: string) -> string {
    parts := split(name, ".");
    b: String_Builder;

    for parts {
        if it_index append(*b, ".");
        append(*b, proto_jai_name(it));
    }

    return builder_to_string(*b);
}

proto_upper :: (name: string) -> string {
    bytes := copy_string(name);
    for *bytes if it.* >= #char "a" && it.* <= #char "z" it.* -= 32;
    return bytes;
}

proto_snake_upper :: (name: string) -> string {
    b: String_Builder;

    for name {
        c := it;
        if it_index && c >= #char "A" && c <= #char "Z" {
            prev := name[it_index - 1];
            next_lower := it_index + 1 < name.count && name[it_index + 1] >= #char "a" && name[it_index + 1] <= #char "z";
            if (prev >= #char "a" && prev <= #char "z") || proto_digit(prev) || (prev >= #char "A" && prev <= #char "Z" && next_lower) append(*b, "_");
        }

        if c >= #char "a" && c <= #char "z" c -= 32;

        append(*b, string.{ 1, *c });
    }

    return builder_to_string(*b);
}

proto_find_type :: (s: *proto_Schema, initial_scope: string, input_name: string, source: *proto_Source) -> *proto_Node {
    scope := initial_scope;
    name := input_name;

    if begins_with(name, ".") {
        name = slice(name, 1, name.count - 1);
        found := proto_symbols.find(*s.names, name);
        if found && !found.*.synthetic && proto_visible(source, found.*.source) return found.*;

        return null;
    }

    while true {
        full := proto_join_name(scope, name);
        found := proto_symbols.find(*s.names, full);

        if found && !found.*.synthetic && proto_visible(source, found.*.source) return found.*;

        if !scope.count break;

        last: s64 = -1;
        for scope if it == #char "." last = it_index;
        if last < 0 scope = "";
        else scope = slice(scope, 0, last);
    }

    return null;
}

proto_field_type :: (f: *proto_Field) -> string {
    if f.target {
        if f.target.kind == .MESSAGE {
            if f.repeated return f.target.jai;
            return proto_schema_format("*%", f.target.jai);
        }

        return f.target.jai;
    }

    for proto_SCALARS if it == f.scalar return proto_JAI_TYPES[it_index];

    return "s32";
}

proto_field_kind :: (f: *proto_Field) -> string {
    if f.wire_kind.count return f.wire_kind;

    if proto_is_message(f) {
        if f.features.message == "DELIMITED" return "GROUP";
        return "MESSAGE";
    }

    if f.target && f.target.features.enums == "CLOSED" return "CLOSED_ENUM";
    if f.scalar == "string" && f.features.utf8 == "NONE" return "RAW_STRING";
    if f.target return "ENUM";

    return proto_upper(f.scalar);
}

proto_is_message :: (f: *proto_Field) -> bool {
    return f.target && f.target.kind == .MESSAGE;
}

proto_packable :: (f: *proto_Field) -> bool {
    return !proto_is_message(f) && f.scalar != "string" && f.scalar != "bytes" && !f.map_entry;
}

proto_field_wire :: (f: *proto_Field) -> s64 {
    if proto_is_message(f) && f.features.message == "DELIMITED" return 3;
    if proto_is_message(f) || f.map_entry || f.scalar == "string" || f.scalar == "bytes" return 2;
    if f.scalar == "fixed64" || f.scalar == "sfixed64" || f.scalar == "double" return 1;
    if f.scalar == "fixed32" || f.scalar == "sfixed32" || f.scalar == "float" return 5;

    return 0;
}

proto_resolve_schema :: (s: *proto_Schema) { // jailint: allow(cognitive_complexity)
    proto_resolve_features(s);

    if s.options.map == .CUSTOM {
        if !s.options.map_module.count || !s.options.map_type.count || !s.options.map_insert.count || !s.options.map_deinit.count {
            proto_schema_fail(s, .{ path = s.options.path, line = 1, column = 1 }, "CUSTOM maps require MAP_MODULE, MAP_TYPE, MAP_INSERT and MAP_DEINIT");
            return;
        }
    }

    // These names are the public generator API and module parameters in this same scope.
    reserved_api :: string.["Map_Mode", "Generate_Options", "Schema_Error", "protobuf_generate", "PATH", "TEXT", "DIR", "IMPORT_PATHS", "MAP", "MAP_MODULE", "MAP_TYPE", "MAP_INSERT", "MAP_DEINIT", "STRIP_ENUM_PREFIX", "proto_registry"];

    // Every package and each of its dotted ancestors, mapped to the first root declared under it.
    package_roots: proto_symbols.Map(string, *proto_Node);

    for root: s.roots {
        package := root.package;

        while package.count {
            first, added := proto_symbols.entry(*package_roots, package);
            if added first.* = root;

            last: s64 = -1;
            for package if it == #char "." last = it_index;
            if last < 0 break;

            package.count = last;
        }
    }

    for root: s.roots {
        root_package := root.package;
        for root_package if it == #char "." {
            root_package = slice(root_package, 0, it_index);
            break;
        }

        for reserved_api {
            if root_package == it proto_schema_fail(s, root.token, "package name is reserved by the Schema module API");
            if !root.package.count && root.name == it proto_schema_fail(s, root.token, "type name is reserved by the Schema module API");
        }

        other := proto_symbols.find(*package_roots, root.full);
        if other proto_schema_fail(s, other.*.token, "package and type have the same name");
    }

    for node: s.nodes {
        node.jai = proto_qualified_jai(node.full);
        if proto_symbols.find(*s.names, node.full) || proto_symbols.find(*s.jai_names, node.jai) proto_schema_fail(s, node.token, "duplicate type name after Jai renaming");
        proto_symbols.insert(*s.names, node.full, node);
        proto_symbols.insert(*s.jai_names, node.jai, node);
    }

    for node: s.nodes {

        if node.kind == .ENUM {
            if !node.values.count || (node.features.enums == "OPEN" && node.values[0].value != 0) proto_schema_fail(s, node.token, "the first proto3 enum value must be zero");

            prefix := proto_schema_format("%_", proto_snake_upper(node.name));

            for *v: node.values {
                original_name := v.name;

                if s.options.strip_enum_prefix && begins_with(v.name, prefix) && v.name.count > prefix.count && !proto_digit(v.name[prefix.count]) {
                    stripped := slice(v.name, prefix.count, v.name.count - prefix.count);

                    // Keep the full name when another value already uses the short one (aliases often do).
                    taken := false;

                    for other: node.values if other.name == stripped taken = true;
                    if !taken v.name = stripped;
                }

                v.name = proto_jai_name(v.name);
                if v.value < -2147483648 || v.value > 2147483647 proto_schema_fail(s, v.token, "enum value exceeds s32");

                for other: node.values {
                    if other.token.line == v.token.line && other.token.column == v.token.column break;
                    if other.name == v.name proto_schema_fail(s, v.token, "duplicate enum value name");
                    if other.value == v.value && !node.allow_alias proto_schema_fail(s, v.token, "duplicate enum number requires allow_alias");
                }

                for node.reserved if v.value >= it.first && v.value <= it.last proto_schema_fail(s, v.token, "reserved enum number is used");
                for node.reserved_names if original_name == it proto_schema_fail(s, v.token, "reserved enum name is used");

            }

            continue;
        }

        for name: node.oneofs {
            for i: 0..it_index - 1 if proto_jai_name(node.oneofs[i]) == proto_jai_name(name) proto_schema_fail(s, node.token, "duplicate oneof name after Jai renaming");
            for child: node.children if proto_jai_name(child.name) == proto_jai_name(name) proto_schema_fail(s, child.token, "oneof and nested type have the same name");
        }

        for f: node.fields {
            f.jai = proto_jai_name(f.name);
            if f.oneof.count f.jai = proto_oneof_name(f.name);
            if f.number <= 0 || f.number > 536870911 || (f.number >= 19000 && f.number <= 19999) proto_schema_fail(s, f.token, "invalid or reserved field number");

            for other: node.fields {
                if other == f break;
                if other.number == f.number proto_schema_fail(s, f.token, "duplicate field number");
                if other.name == f.name || other.jai == f.jai proto_schema_fail(s, f.token, "duplicate field name after Jai renaming");
            }

            for node.reserved if f.number >= it.first && f.number <= it.last proto_schema_fail(s, f.token, "reserved field number is used");
            for node.reserved_names if f.name == it proto_schema_fail(s, f.token, "reserved field name is used");

            for name: node.oneofs {
                if f.name == name proto_schema_fail(s, f.token, "field and oneof have the same name");
            }

            for child: node.children if proto_jai_name(child.name) == f.jai proto_schema_fail(s, f.token, "field and nested type have the same name");

            if f.map_entry {
                if f.explicit_packed proto_schema_fail(s, f.token, "packed does not apply to maps");
                continue;
            }

            for proto_SCALARS if it == f.type_name f.scalar = it;

            if !f.scalar.count {
                f.target = proto_find_type(s, node.full, f.type_name, node.source);
                if !f.target proto_schema_fail(s, f.token, proto_schema_format("unknown type '%'", f.type_name));
            }

            if f.has_default && (f.repeated || f.oneof.count || proto_is_message(f) || !f.optional) proto_schema_fail(s, f.token, "defaults require explicit scalar presence");

            if f.target && f.target.kind == .ENUM && f.has_default {
                found := false;

                for f.target.values if it.token.text == f.default_value {
                    f.default_value = proto_schema_format("cast(%) %", proto_field_type(f), it.value);
                    found = true;
                    break;
                }

                if !found proto_schema_fail(s, f.token, "unknown enum default");
            }

            f.wire_kind = proto_field_kind(f);
            if node.synthetic && f.number == 1 && (f.target || f.scalar == "float" || f.scalar == "double" || f.scalar == "bytes") proto_schema_fail(s, f.token, "invalid map key type");
            if f.explicit_packed && (!f.repeated || !proto_packable(f)) proto_schema_fail(s, f.token, "packed applies only to repeated scalar or enum fields");

            if f.oneof.count {

                for other: node.fields {
                    if other == f break;
                    if other.oneof == f.oneof && proto_oneof_tag(proto_oneof_name(other.name)) == proto_oneof_tag(f.jai) proto_schema_fail(s, f.token, "duplicate oneof tag after Jai renaming");
                }
            }
        }
    }

    changed := true;
    while changed {
        changed = false;

        for node: s.nodes {
            if node.has_required continue;

            for f: node.fields {
                target := f.target;
                if f.map_entry target = f.map_entry;

                if f.required || (target && target.has_required) {
                    node.has_required = true;
                    changed = true;
                    break;
                }
            }
        }
    }

}

// Variant reserves its layout members and strips one trailing underscore from tags.
proto_oneof_tag :: (name: string) -> string {
    text := name;
    if text.count && text[text.count - 1] == #char "_" text.count -= 1;
    return proto_upper(text);
}

proto_oneof_name :: (name: string) -> string {
    result := proto_jai_name(name);
    reserved :: string.["which", "Which", "payload", "types", "none", "case_count", "case_names", "case_infos", "payload_size", "payload_align"];

    for reserved if result == it {
        result = proto_schema_format("%_", result);
        break;
    }

    if begins_with(result, "case_type_") result = proto_schema_format("%_", result);
    if proto_oneof_tag(result) == "NONE" result = proto_schema_format("%_", result);

    return result;
}
