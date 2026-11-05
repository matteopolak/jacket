#scope_module

proto_emit_schema :: (s: *proto_Schema, b: *String_Builder) {
    append(b, "// Generated from protobuf by Extensions/Protobuf/Schema.\n");
    append(b, "#scope_module\nproto_b :: #import \"Basic\";\nproto_r :: #import \"Extensions/Protobuf\";\nproto_v :: #import \"Extensions/Variant\";\n");
    if s.options.map == .HASH_MAP append(b, "proto_m :: #import \"Extensions/Hash_Map\";\n");
    if s.options.map == .TABLE append(b, "proto_m :: #import \"Hash_Table\";\n");
    if s.options.map == .CUSTOM proto_emit(b, "proto_m :: #import \"%\";\n", s.options.map_module);

    if s.options.grpc append(b, "#assert OS != .WASM \"generated gRPC services require native Io\";\nproto_g :: #import \"Extensions/Grpc\";\nproto_h :: #import \"Extensions/Http\";\nproto_i :: #import \"Extensions/Io\";\n");

    append(b, "#scope_export\n");
    proto_emit_packages(s, b);
    proto_emit_registry(s, b);
}

// A package level: its own types and nested packages, in order of first declaration.
proto_Namespace :: struct {
    name: string;
    items: [..] proto_Namespace_Item;
}

proto_Namespace_Item :: struct {
    node: *proto_Node;
    child: *proto_Namespace;
    service: *proto_Service;
}

proto_emit_packages :: (s: *proto_Schema, b: *String_Builder) {
    top: proto_Namespace;
    levels: proto_symbols.Map(string, *proto_Namespace);

    entries: [..] proto_Namespace_Item;

    for s.roots array_add(*entries, .{ node = it });
    for s.services array_add(*entries, .{ service = it });

    for entry: entries {
        package := ifx entry.node then entry.node.package else entry.service.package;
        level := *top;
        rest := package;

        while rest.count {
            part := rest;

            for rest if it == #char "." {
                part.count = it_index;
                break;
            }

            // Package text is shared by every root of the package, so the prefix is a slice of it.
            path := slice(package, 0, package.count - rest.count + part.count);
            child, added := proto_symbols.entry(*levels, path);

            if added {
                child.* = New(proto_Namespace);
                child.*.name = part;
                array_add(*level.items, .{ child = child.* });
            }

            level = child.*;
            rest = slice(rest, part.count, rest.count - part.count);
            if rest.count rest = slice(rest, 1, rest.count - 1);
        }

        array_add(*level.items, entry);
    }

    proto_emit_namespace(s, b, *top);
}

proto_emit_namespace :: (s: *proto_Schema, b: *String_Builder, level: *proto_Namespace) {
    for level.items {
        if it.service {
            proto_emit_service(s, b, it.service);
            continue;
        }

        if it.node {
            proto_emit_node(s, b, it.node);
            continue;
        }

        proto_emit(b, "% :: struct {\n", proto_jai_name(it.child.name));
        proto_emit_namespace(s, b, it.child);
        append(b, "}\n");
    }
}

proto_map_type :: (s: *proto_Schema, f: *proto_Field) -> string {
    k := proto_field_type(f.map_entry.fields[0]);
    v := proto_field_type(f.map_entry.fields[1]);

    if s.options.map == .ARRAY return proto_schema_format("[..] proto_r.Map_Entry(%, %)", k, v);
    if s.options.map == .HASH_MAP return proto_schema_format("proto_m.Map(%, %)", k, v);
    if s.options.map == .TABLE return proto_schema_format("proto_m.Table(%, %)", k, v);

    return proto_schema_format("proto_m.%(%, %)", s.options.map_type, k, v);
}

proto_emit_node :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    if node.synthetic return;

    if node.kind == .EXTENSION {
        proto_emit_extension(b, node);
        return;
    }

    if node.kind == .ENUM {
        proto_emit(b, "% :: enum s32 {\n", proto_jai_name(node.name));
        for node.values proto_emit(b, "% :: %;\n", it.name, it.value);
        append(b, "}\n");
        return;
    }

    proto_emit(b, "% :: struct {\n", proto_jai_name(node.name));
    for node.children proto_emit_node(s, b, it);
    for f: node.fields if f.map_entry proto_emit(b, "% :: %;\n", proto_entry_type(f), proto_entry_codec(f));

    for node.fields {
        if it.oneof.count continue;

        proto_emit(b, "%: ", it.jai);

        if it.map_entry append(b, proto_map_type(s, it));
        else {
            if it.repeated append(b, "[..] ");
            if it.optional && !proto_is_message(it) proto_emit(b, "proto_r.Optional(%)", proto_field_type(it));
            else append(b, proto_field_type(it));
        }

        proto_emit_default(b, it);
        append(b, ";");
        proto_emit_field_notes(b, it);

        append(b, "\n");
    }

    // JSON prints fields in declaration order, so a oneof member declared before a plain field
    // says where it goes.
    last_plain := -1;
    for node.fields if !it.oneof.count last_plain = it_index;

    for name: node.oneofs {
        proto_emit(b, "%: proto_v.Variant(struct {\nnone: void;\n", proto_jai_name(name));

        for node.fields if it.oneof == name {
            proto_emit(b, "%: %;", it.jai, proto_field_type(it));
            if it_index < last_plain proto_emit(b, " @proto_at_%", it_index);
            proto_emit_field_notes(b, it);
            append(b, "\n");
        }

        append(b, "});\n");
        proto_emit(b, "proto_%_numbers :: s64.[0", name);
        for node.fields if it.oneof == name proto_emit(b, ", %", it.number);

        append(b, "];\n");
    }

    append(b, "_unknown: [..] u8;\nproto_storage: *proto_r.Storage_Block; @json(skip)\n");
    proto_emit_json_support(s, b, node);
    proto_emit_size(s, b, node);
    proto_emit_encode(s, b, node);
    proto_emit_free(s, b, node);
    proto_emit_equal(s, b, node);
    proto_emit_merge(s, b, node);
    proto_emit(b, "proto_has_required :: %;\n", ifx node.has_required then "true" else "false");
    if node.has_required proto_emit_check(s, b, node);

    append(b, "}\n");
}

proto_field_expr :: (f: *proto_Field, base: string = "v") -> string {
    if f.oneof.count return proto_schema_format("%.%.%", base, proto_jai_name(f.oneof), f.jai);
    if f.optional && !proto_is_message(f) return proto_schema_format("%.%.value", base, f.jai);
    return proto_schema_format("%.%", base, f.jai);
}

proto_emit_map_entry :: (s: *proto_Schema, b: *String_Builder, f: *proto_Field, base: string) {
    if s.options.map == .ARRAY {
        proto_emit(b, "for item: %.% {\nentry := %.{key = item.key, value = item.value};\n", base, f.jai, proto_entry_type(f));
    } else proto_emit(b, "for item, key: %.% {\nentry := %.{key = key, value = item};\n", base, f.jai, proto_entry_type(f));
}

proto_emit_size :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    proto_emit(b, "proto_size :: (v: *%, c: *[..] u32) -> s64 {\nslot := c.count;\nproto_b.array_add(c, 0);\nn: s64;\n", node.jai);

    for f: node.fields {
        if f.map_entry && s.options.map != .CUSTOM {
            proto_emit(b, "n += proto_r.size_map(*v.%, %, .%, %, c);\n", f.jai, proto_entry_type(f), proto_map_mode(s), f.number * 8 + 2);
        } else if f.map_entry {
            proto_emit_map_entry(s, b, f, "v");
            proto_emit(b, "n += proto_r.size_field(*entry, .MESSAGE, %, c);\n}\n", f.number * 8 + 2);
        } else proto_emit_field_call(b, f, false);
    }

    append(b, "n = proto_b.min(n + v._unknown.count, 2147483648);\nc.data[slot] = cast(u32) n;\nreturn n;\n}\n");
}

proto_emit_encode :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    proto_emit(b, "proto_encode :: (v: *%, w: *proto_r.Writer) {\n", node.jai);

    for f: node.fields {
        if f.map_entry && s.options.map != .CUSTOM {
            proto_emit(b, "proto_r.encode_map(w, *v.%, %, .%, %);\n", f.jai, proto_entry_type(f), proto_map_mode(s), f.number * 8 + 2);
        } else if f.map_entry {
            proto_emit_map_entry(s, b, f, "v");
            proto_emit(b, "proto_r.encode_field(w, *entry, .MESSAGE, %);\n}\n", f.number * 8 + 2);
        } else proto_emit_field_call(b, f, true);
    }

    append(b, "proto_r.write_bytes(w, v._unknown);\n}\n");
}

proto_emit_field_call :: (b: *String_Builder, f: *proto_Field, writing: bool) {
    prefix := "n += proto_r.size_";
    suffix := ", c";

    if writing {
        prefix = "proto_r.encode_";
        suffix = "";
    }

    argument := proto_field_expr(f);
    if writing argument = proto_schema_format("w, %", argument);

    mode := "field";
    tag := f.number * 8 + proto_field_wire(f);

    if f.repeated {
        mode = "repeated";
        packed := "false";

        if proto_packable(f) && f.packed {
            packed = "true";
            tag = f.number * 8 + 2;
        }

        suffix = proto_schema_format(", %%", packed, suffix);
    } else if f.optional && !proto_is_message(f) {
        mode = "optional";
        argument = proto_schema_format("v.%", f.jai);
        if writing argument = proto_schema_format("w, %", argument);
    }

    if f.oneof.count {
        proto_emit(b, "if v.%.which == .% ", proto_jai_name(f.oneof), proto_oneof_tag(f.jai));
        suffix = proto_schema_format("%, true", suffix);
    }

    proto_emit(b, "%%(%, .%, %%);\n", prefix, mode, argument, proto_field_kind(f), tag, suffix);
}

proto_entry_type :: (f: *proto_Field) -> string {
    return proto_schema_format("proto_entry_%", f.number);
}

proto_entry_codec :: (f: *proto_Field) -> string {
    k := f.map_entry.fields[0];
    v := f.map_entry.fields[1];
    return proto_schema_format("proto_r.Entry_Codec(%, %, .%, .%)", proto_field_type(k), proto_field_type(v), proto_field_kind(k), proto_field_kind(v));
}

proto_emit_item_free :: (b: *String_Builder, f: *proto_Field, expr: string) {
    if proto_is_message(f) || f.scalar == "string" || f.scalar == "bytes" proto_emit(b, "proto_r.free_field(*%, .%);\n", expr, proto_field_kind(f));
}

proto_emit_free :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    proto_emit(b, "proto_free :: proto_r.free_message;\nproto_release :: (v: *%) {\n", node.jai);
    for name: node.oneofs proto_emit(b, "proto_r.free_oneof(*v.%);\n", proto_jai_name(name));

    for f: node.fields {
        if f.oneof.count continue;

        if f.map_entry && s.options.map != .CUSTOM {
            proto_emit(b, "proto_r.free_map(*v.%, %, .%);\n", f.jai, proto_entry_type(f), proto_map_mode(s));
        } else if f.map_entry {
            proto_emit_map_entry(s, b, f, "v");
            proto_emit(b, "%.proto_free(*entry);\n}\n", proto_entry_type(f));
            if s.options.map == .ARRAY proto_emit(b, "proto_b.array_free(v.%);\n", f.jai);
            else if s.options.map == .HASH_MAP proto_emit(b, "proto_m.deinit(*v.%);\n", f.jai);
            else if s.options.map == .TABLE proto_emit(b, "proto_m.table_reset(*v.%);\n", f.jai);
            else proto_emit(b, "proto_m.%(*v.%);\n", s.options.map_deinit, f.jai);
        } else if f.repeated {
            proto_emit(b, "proto_r.free_repeated(*v.%, .%);\n", f.jai, proto_field_kind(f));
        } else if proto_is_message(f) || f.scalar == "string" || f.scalar == "bytes" {
            if f.optional && !proto_is_message(f) proto_emit(b, "if v.%.has {\n", f.jai);
            proto_emit_item_free(b, f, proto_field_expr(f));
            if f.optional && !proto_is_message(f) append(b, "}\n");
        }
    }

    append(b, "proto_b.array_free(v._unknown);\nv.* = .{};\n}\n");
}

proto_emit_item_equal :: (b: *String_Builder, f: *proto_Field, a: string, c: string) {
    proto_emit(b, "if !proto_r.equal_field(%, %, .%) return false;\n", a, c, proto_field_kind(f));
}

proto_emit_equal :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    if s.options.map == .CUSTOM {
        proto_emit_equal_custom(s, b, node);
        return;
    }

    proto_emit(b, "proto_equal :: (a: *%, b: *%) -> bool {\nreturn true", node.jai, node.jai);
    for name: node.oneofs proto_emit(b, "\n&& proto_r.equal_oneof(*a.%, *b.%)", proto_jai_name(name), proto_jai_name(name));

    for f: node.fields {
        if f.oneof.count continue;

        if f.map_entry proto_emit(b, "\n&& proto_r.equal_map(*a.%, *b.%, %, .%)", f.jai, f.jai, proto_entry_type(f), proto_map_mode(s));
        else {
            mode := "field";
            if f.repeated mode = "repeated";
            if f.optional && !proto_is_message(f) mode = "optional";
            proto_emit(b, "\n&& proto_r.equal_%(a.%, b.%, .%)", mode, f.jai, f.jai, proto_field_kind(f));
        }
    }

    append(b, "\n&& proto_r.bytes_equal(a._unknown, b._unknown);\n}\n");
}

proto_emit_equal_custom :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    proto_emit(b, "proto_equal :: (a: *%, b: *%) -> bool {\n", node.jai, node.jai);
    for name: node.oneofs proto_emit(b, "if !proto_r.equal_oneof(*a.%, *b.%) return false;\n", proto_jai_name(name), proto_jai_name(name));

    for f: node.fields {
        if f.oneof.count continue;

        if f.map_entry && s.options.map != .CUSTOM {
            proto_emit(b, "if !proto_r.equal_map(*a.%, *b.%, %, .%) return false;\n", f.jai, f.jai, proto_entry_type(f), proto_map_mode(s));
        } else if f.map_entry {
            proto_emit(b, "if a.%.count != b.%.count return false;\n", f.jai, f.jai);

            if s.options.map == .ARRAY {
                proto_emit(b, "for item: a.% {\nother := b.%[it_index];\n", f.jai, f.jai);
                proto_emit_item_equal(b, f.map_entry.fields[0], "item.key", "other.key");
                proto_emit_item_equal(b, f.map_entry.fields[1], "item.value", "other.value");
                append(b, "}\n");
            } else if s.options.map == .HASH_MAP || s.options.map == .TABLE {
                proto_emit(b, "for item, key: a.% {\n", f.jai);
                if s.options.map == .HASH_MAP proto_emit(b, "p := proto_m.find(*b.%, key);\n", f.jai);
                else proto_emit(b, "p := proto_m.table_find_pointer(*b.%, key);\n", f.jai);

                append(b, "if !p return false;\nother := p.*;\n");
                proto_emit_item_equal(b, f.map_entry.fields[1], "item", "other");
                append(b, "}\n");
            } else {
                proto_emit(b, "for item, key: a.% {\nfound := false;\nfor other, other_key: b.% {\nif key == other_key {\nfound = true;\n", f.jai, f.jai);
                proto_emit_item_equal(b, f.map_entry.fields[1], "item", "other");
                append(b, "break;\n}\n}\nif !found return false;\n}\n");
            }
        } else {
            mode := "field";
            if f.repeated mode = "repeated";
            if f.optional && !proto_is_message(f) mode = "optional";
            if f.oneof.count proto_emit(b, "if a.%.which == .% ", proto_jai_name(f.oneof), proto_oneof_tag(f.jai));

            a := proto_schema_format("a.%", f.jai);
            c := proto_schema_format("b.%", f.jai);

            if f.oneof.count {
                a = proto_field_expr(f, "a");
                c = proto_field_expr(f, "b");
            }

            proto_emit(b, "if !proto_r.equal_%(%, %, .%) return false;\n", mode, a, c, proto_field_kind(f));
        }
    }

    append(b, "return proto_r.bytes_equal(a._unknown, b._unknown);\n}\n");
}

proto_emit_read_value :: (b: *String_Builder, f: *proto_Field, expr: string, path: string) {
    if proto_is_message(f) {
        reader := "read_message";
        if f.features.message == "DELIMITED" reader = "read_group";
        proto_emit(b, "if !% % = proto_b.New(%);\nif !proto_r.%(r, %) return proto_r.field_error(r, \"%\");\n", expr, expr, f.target.jai, reader, expr, path);
    } else proto_emit(b, "if !proto_r.read_scalar(r, *%, .%) return proto_r.field_error(r, \"%\");\n", expr, proto_field_kind(f), path);
}

proto_emit_read_field :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node, f: *proto_Field) {
    proto_emit(b, "#if field == % {\n", f.number);
    if proto_field_kind(f) == "CLOSED_ENUM" || proto_field_kind(f) == "GROUP" proto_emit(b, "r.enum_tag = %;\n", f.number * 8);

    if !f.oneof.count && (!f.map_entry || s.options.map != .CUSTOM) {
        mode := "field";
        if f.repeated mode = "repeated";
        if f.optional && !proto_is_message(f) mode = "optional";
        if f.map_entry proto_emit(b, "return proto_r.read_map(r, *v.%, %, .%, wire, \"%\");\n}\n", f.jai, proto_entry_type(f), proto_map_mode(s), f.name);
        else proto_emit(b, "return proto_r.read_%(r, *v.%, .%, wire, \"%\");\n}\n", mode, f.jai, proto_field_kind(f), f.name);

        return;
    }

    proto_emit(b, "if wire != % return false;\n", proto_field_wire(f));

    if f.map_entry {
        proto_emit(b, "entry: %;\ndefer proto_r.release_entry(r, *entry);\nif !proto_r.read_message(r, *entry) return proto_r.field_error(r, \"%\");\n", proto_entry_type(f), f.name);

        if s.options.map == .ARRAY {
            proto_emit(b, "proto_b.array_add(*v.%, .{entry.key, entry.value});\nentry.key = .{}; entry.value = .{};\n", f.jai);
        } else {
            proto_emit(b, "for old, key: v.% {\nif key == entry.key {\nprevious := old;\n", f.jai);
            proto_emit(b, "proto_r.release_field(r, *previous, .%);\nproto_r.release_field(r, *entry.key, .%);\n", proto_field_kind(f.map_entry.fields[1]), proto_field_kind(f.map_entry.fields[0]));
            append(b, "entry.key = key;\n");
            proto_emit_map_insert(s, b, f);
            append(b, "entry.key = .{}; entry.value = .{};\nreturn true;\n}\n}\n");
            proto_emit_map_insert(s, b, f);

            append(b, "entry.key = .{}; entry.value = .{};\n");
        }
    } else if f.repeated {
        proto_emit(b, "item: %;\n", proto_field_type(f));
        // A failing nested item may own a partially decoded message.
        append(b, "ok := true;\n");
        if proto_is_message(f) append(b, "ok = proto_r.read_message(r, *item);\n");
        else proto_emit(b, "ok = proto_r.read_scalar(r, *item, .%);\n", proto_field_kind(f));

        append(b, "if !ok {\n");
        if proto_is_message(f) proto_emit(b, "%.proto_free(*item);\n", f.target.jai);
        else proto_emit_item_free(b, f, "item");

        proto_emit(b, "return proto_r.field_error(r, \"%\", v.%.count);\n}\nproto_b.array_add(*v.%, item);\n", f.name, f.jai, f.jai);
    } else {
        if f.oneof.count {
            if proto_field_kind(f) == "CLOSED_ENUM" {
                proto_emit(b, "item: %;\nif !proto_r.read_scalar(r, *item, .CLOSED_ENUM) return proto_r.field_error(r, \"%\");\nif r.enum_rejected return true;\n", proto_field_type(f), f.name);
            }

            proto_emit(b, "if v.%.which != .% {\nproto_clear_%(v);\nv.%.which = .%;\n}\n", proto_jai_name(f.oneof), proto_oneof_tag(f.jai), f.oneof, proto_jai_name(f.oneof), proto_oneof_tag(f.jai));
        }

        if f.oneof.count && proto_field_kind(f) == "CLOSED_ENUM" proto_emit(b, "% = item;\n", proto_field_expr(f));
        else proto_emit_read_value(b, f, proto_field_expr(f), f.name);
        if f.optional && !proto_is_message(f) proto_emit(b, "v.%.has = true;\n", f.jai);
    }

    append(b, "return true;\n}\n");
}

// Dispatch on complete tags so the hot path validates wire type and selects the field once.
proto_emit_merge :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    special := false;
    for node.fields if (it.map_entry && s.options.map == .CUSTOM) special = true;

    if special {
        proto_emit(b, "proto_read :: inline (v: *%, r: *proto_r.Reader, $field: s64, wire: u64) -> bool {\n", node.jai);
        for node.fields if (it.map_entry && s.options.map == .CUSTOM) proto_emit_read_field(s, b, node, it);
        append(b, "return false;\n}\n");
    }

    proto_emit(b, "proto_merge :: (v: *%, r: *proto_r.Reader) -> bool {\nold_unknown := r.unknown;\nr.unknown = *v._unknown;\ndefer r.unknown = old_unknown;\nwhile proto_r.remaining(r) > 0 {\nr.tag_start = r.pos;\ntag := proto_r.read_tag(r);\nif r.error.kind != .NONE return false;\nif tag & 7 == 4 return proto_r.end_group(r, tag);\nif tag == {\n", node.jai);

    for f: node.fields {
        proto_emit(b, "case %", f.number * 8 + proto_field_wire(f));
        if f.repeated && proto_packable(f) proto_emit(b, ", %", f.number * 8 + 2);

        append(b, ";\n");

        if f.map_entry && s.options.map == .CUSTOM {
            proto_emit(b, "if !proto_read(v, r, %, tag & 7) return false;\n", f.number);
        } else if f.oneof.count {
            if proto_field_kind(f) == "CLOSED_ENUM" || proto_field_kind(f) == "GROUP" proto_emit(b, "r.enum_tag = %;\n", f.number * 8);
            proto_emit(b, "if !proto_r.read_oneof(r, *v.%, .%, .%, tag & 7, \"%\") return false;\n", proto_jai_name(f.oneof), proto_oneof_tag(f.jai), proto_field_kind(f), f.name);
        } else {
            if proto_field_kind(f) == "CLOSED_ENUM" || proto_field_kind(f) == "GROUP" proto_emit(b, "r.enum_tag = %;\n", f.number * 8);

            mode := "field";
            if f.repeated mode = "repeated";
            if f.optional && !proto_is_message(f) mode = "optional";
            if f.map_entry proto_emit(b, "if !proto_r.read_map(r, *v.%, %, .%, tag & 7, \"%\") return false;\n", f.jai, proto_entry_type(f), proto_map_mode(s), f.name);
            else proto_emit(b, "if !proto_r.read_%(r, *v.%, .%, tag & 7, \"%\") return false;\n", mode, f.jai, proto_field_kind(f), f.name);
        }

        append(b, "continue;\n");
    }

    if node.checks_extensions append(b, "}\nif !proto_skip_unknown(r, tag) return false;\n}\nreturn true;\n}\n");
    else append(b, "}\nif !proto_r.skip_unknown(r, tag) return false;\n}\nreturn true;\n}\n");
    if node.checks_extensions proto_emit_extension_checks(b, node);
}

// Keeps an unmatched field, then validates it if it is a known extension holding a checked string.
proto_emit_extension_checks :: (b: *String_Builder, node: *proto_Node) {
    append(b, "proto_skip_unknown :: (r: *proto_r.Reader, tag: u64) -> bool {\nif !proto_r.skip_unknown(r, tag) return false;\nnumber := tag >> 3;\n");

    for e: node.extensions {
        f := e.fields[0];
        if f.wire_kind != "STRING" && !(proto_is_message(f) && f.target.verifies) continue;

        element := "string";
        if f.target element = f.target.jai;

        proto_emit(b, "if number == % return proto_r.check_extension(r, tag, .%, %, %);\n", f.number, proto_field_kind(f), element, proto_quote(e.full));
    }

    append(b, "return true;\n}\n");
}

// Specialize literal templates once, so generating a schema does not scan every output byte.
// Arguments remain a small string/s64-only formatter, rather than the reflective Basic printer.
proto_emit :: (b: *String_Builder, $template: string, args: ..Any) {
    #insert #run proto_format_code(template, "out := cast(*u8) ensure_contiguous_space(b, n);\nif !out return;\n");
    advance_through_ensured_space(b, n);
}

// Exact allocation for small internal names/expressions, avoiding a 4 KiB builder per name.
proto_schema_format :: ($template: string, args: ..Any) -> string {
    #insert #run proto_format_code(template, "out := cast(*u8) alloc(n);\n");
    return .{ n, out };
}

// Code that sizes the formatted text as `n`, runs `destination` to get `out`, then fills it.
// Each '%' takes the next string or s64 argument.
proto_format_code :: (template: string, destination: string) -> string {
    b: String_Builder;
    literals := template.count;
    count: s64;

    for template if it == #char "%" {
        literals -= 1;
        count += 1;
    }

    print(*b, "n := %;\n", literals);
    for i: 0..count - 1 print(*b, "n += proto_argument_size(args[%]);\n", i);

    append(*b, destination);
    append(*b, "pos: s64;\n");

    start, argument: s64;

    for template {
        if it != #char "%" continue;

        literal := slice(template, start, it_index - start);
        if literal.count print(*b, "memcpy(out + pos, %.data, %); pos += %;\n", proto_quote(literal,, temp), literal.count, literal.count);

        print(*b, "pos += proto_argument_write(out + pos, args[%]);\n", argument);
        argument += 1;
        start = it_index + 1;
    }

    literal := slice(template, start, template.count - start);
    if literal.count print(*b, "memcpy(out + pos, %.data, %);\n", proto_quote(literal,, temp), literal.count);

    print(*b, "assert(args.count == %);\n", argument);

    return builder_to_string(*b);
}

proto_argument_size :: (arg: Any) -> s64 {
    if arg.type.type == .STRING return (cast(*string) arg.value_pointer).count;

    assert(arg.type.type == .INTEGER && arg.type.runtime_size == 8);
    v := (cast(*s64) arg.value_pointer).*;
    n: s64 = 1;

    if v < 0 {
        n += 1;
        v = -v;
    }

    while v >= 10 {
        n += 1;
        v /= 10;
    }

    return n;
}

proto_argument_write :: (out: *u8, arg: Any) -> s64 {
    if arg.type.type == .STRING {
        text := (cast(*string) arg.value_pointer).*;
        if text.count memcpy(out, text.data, text.count);
        return text.count;
    }

    n := proto_argument_size(arg);
    v := (cast(*s64) arg.value_pointer).*;

    if v < 0 {
        out[0] = #char "-";
        v = -v;
    }

    pos := n;

    while true {
        pos -= 1;
        out[pos] = cast(u8) (v % 10) + #char "0";
        v /= 10;
        if !v break;
    }

    return n;
}

proto_emit_map_insert :: (s: *proto_Schema, b: *String_Builder, f: *proto_Field) {
    if s.options.map == .HASH_MAP proto_emit(b, "proto_m.insert(*v.%, entry.key, entry.value);\n", f.jai);
    else if s.options.map == .TABLE proto_emit(b, "proto_m.table_set(*v.%, entry.key, entry.value);\n", f.jai);
    else proto_emit(b, "proto_m.%(*v.%, entry.key, entry.value);\n", s.options.map_insert, f.jai);
}

// A Jai string literal, or a JSON string inside a Json note. Both escape control bytes.
proto_quote :: (text: string, json := false) -> string {
    b: String_Builder;

    append(*b, "\"");

    for text {
        if it == 34 append(*b, "\\\"");
        else if it == 92 append(*b, "\\\\");
        else if it < 32 || (!json && it >= 127) {
            if json append(*b, "\\u00");
            else append(*b, "\\x");

            digits :: "0123456789abcdef";
            a := digits[it / 16];
            c := digits[it % 16];

            append(*b, string.{ 1, *a });
            append(*b, string.{ 1, *c });
        } else append(*b, string.{ 1, *it });
    }

    append(*b, "\"");

    return builder_to_string(*b);
}

proto_emit_field_notes :: (b: *String_Builder, f: *proto_Field) {
    if f.deprecated append(b, " @deprecated");

    proto_emit_json_notes(b, f);

    if f.has_json_name {
        // Quote the whole note so arbitrary json_name text cannot become Jai source.
        note := proto_schema_format("json(rename=%)", proto_quote(f.json_name, true));
        proto_emit(b, " @%", proto_quote(note));
    }
}

proto_map_mode :: (s: *proto_Schema) -> string {
    if s.options.map == .HASH_MAP return "HASH_MAP";
    if s.options.map == .TABLE return "TABLE";
    return "ARRAY";
}

// Schema defaults live in the Optional payload; presence remains false.
proto_emit_default :: (b: *String_Builder, f: *proto_Field) {
    value := proto_default_value(f);
    if value.count proto_emit(b, " = .{value = %}", value);
}

// The Jai expression for a scalar field's default, or empty when the zero value is right.
proto_default_value :: (f: *proto_Field) -> string {
    if !f.optional || proto_is_message(f) || f.repeated || f.map_entry return "";

    value := f.default_value;

    if !f.has_default {
        if !f.target || !f.target.values.count return "";
        value = proto_schema_format("cast(%) %", proto_field_type(f), f.target.values[0].value);
    } else if f.default_string {
        value = proto_quote(value);
        if f.scalar == "bytes" value = proto_schema_format(".{%, cast(*u8) %.data}", f.default_value.count, value);
    } else if value == "inf" value = proto_schema_format("cast(%) #run proto_r.float_default(false, false)", proto_field_type(f));
    else if value == "-inf" value = proto_schema_format("cast(%) #run proto_r.float_default(true, false)", proto_field_type(f));
    else if value == "nan" value = proto_schema_format("cast(%) #run proto_r.float_default(false, true)", proto_field_type(f));

    return value;
}

// One extension constant. The extendee, element type and kind are type arguments, so the
// runtime helpers are shared and the constant stays a line of data.
proto_emit_extension :: (b: *String_Builder, node: *proto_Node) {
    f := node.fields[0];
    element := proto_field_type(f);

    if f.target element = f.target.jai;

    mode := "Extension";
    if f.repeated mode = "Repeated_Extension";

    proto_emit(b, "% :: proto_r.%(%, %, .%).{number = %, name = %", proto_jai_name(node.name), mode, f.extendee.jai, element, proto_field_kind(f), f.number, proto_quote(node.full));
    if f.repeated && proto_packable(f) && f.packed append(b, ", packed = true");

    value := proto_default_value(f);
    if value.count proto_emit(b, ", default = %", value);

    append(b, "};\n");
}

proto_emit_check :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    proto_emit(b, "proto_check :: (v: *%, r: *proto_r.Reader) -> bool {\n", node.jai);

    for f: node.fields {
        if f.required {
            test := proto_schema_format("v.%.has", f.jai);
            if proto_is_message(f) test = proto_schema_format("v.% != null", f.jai);
            proto_emit(b, "if !(%) { proto_r.fail(r, .MISSING_REQUIRED); return proto_r.field_error(r, \"%\"); }\n", test, f.name);
        }

        if f.map_entry && proto_is_message(f.map_entry.fields[1]) {
            if s.options.map == .ARRAY proto_emit(b, "for v.% if !proto_r.check_message(it.value, r) return proto_r.field_error(r, \"%\");\n", f.jai, f.name);
            else proto_emit(b, "for v.% if !proto_r.check_message(it, r) return proto_r.field_error(r, \"%\");\n", f.jai, f.name);
        } else if proto_is_message(f) {
            if f.oneof.count proto_emit(b, "if v.%.which == .% ", proto_jai_name(f.oneof), proto_oneof_tag(f.jai));
            if f.repeated proto_emit(b, "for item: v.% if !proto_r.check_message(*item, r) return proto_r.field_error(r, \"%\", it_index);\n", f.jai, f.name);
            else proto_emit(b, "if !proto_r.check_message(%, r) return proto_r.field_error(r, \"%\");\n", proto_field_expr(f), f.name);
        }
    }

    append(b, "return true;\n}\n");
}

// What the JSON mapping can't read from a field's Jai type: its .proto name when Jai renamed
// it, and for enums whether they are closed, NullValue, or have renamed values.
proto_emit_json_notes :: (b: *String_Builder, f: *proto_Field) {
    if f.jai != f.name proto_emit(b, " @%", proto_quote(proto_schema_format("proto_name=%", f.name)));

    target := f.target;
    if f.map_entry target = f.map_entry.fields[1].target;
    if !target || target.kind != .ENUM return;

    if target.full == "google.protobuf.NullValue" append(b, " @proto_null");
    if target.features.enums == "CLOSED" append(b, " @proto_closed");

    renamed := false;
    for target.values if it.token.text != it.name renamed = true;
    if !renamed return;

    names: String_Builder;

    append(*names, "proto_enum=");

    for target.values {
        if it_index append(*names, ",");
        append(*names, it.token.text);
    }

    proto_emit(b, " @%", proto_quote(builder_to_string(*names)));
}

// Well-known types are marked for their special JSON forms; a CUSTOM map gets an insert the
// JSON reader can call.
proto_emit_json_support :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    if begins_with(node.full, "google.protobuf.") {
        name := slice(node.full, 16, node.full.count - 16);
        kind := "";

        if name == "Timestamp" kind = "TIMESTAMP";
        else if name == "Duration" kind = "DURATION";
        else if name == "FieldMask" kind = "FIELD_MASK";
        else if name == "Struct" kind = "STRUCT";
        else if name == "Value" kind = "VALUE";
        else if name == "ListValue" kind = "LIST_VALUE";
        else if name == "Any" kind = "ANY";
        else if name == "Empty" kind = "EMPTY";
        else if name == "DoubleValue" || name == "FloatValue" || name == "Int64Value" || name == "UInt64Value" || name == "Int32Value" || name == "UInt32Value" || name == "BoolValue" || name == "StringValue" || name == "BytesValue" kind = "WRAPPER";

        if kind.count proto_emit(b, "proto_well_known :: proto_r.Well_Known.%;\n", kind);
    }

    // JSON prints and reads the extensions this schema declares as `[full.name]` keys.
    for node.extensions proto_emit(b, "proto_extension_% :: %;\n", it_index, it.jai);

    if s.options.map != .CUSTOM return;

    for node.fields if it.map_entry {
        proto_emit(b, "proto_map_insert :: (m: *$M, k: $K, v: $V) { proto_m.%(m, k, v); }\n", s.options.map_insert);
        break;
    }
}

// Every message of the schema by its full name, for the JSON form of google.protobuf.Any.
// Nothing here is compiled unless a program reads proto_registry.
proto_emit_registry :: (s: *proto_Schema, b: *String_Builder) {
    append(b, "proto_registry :: proto_r.Json_Type.[\n");

    for s.nodes {
        if it.kind != .MESSAGE || it.synthetic continue;
        proto_emit(b, ".{\"%\", proto_r.Json_Any(%).run},\n", it.full, it.jai);
    }

    append(b, "];\n");
}

// Services share the surrounding message graph and emit only typed adapters;
// framing, ownership, cardinality and completion stay in Extensions/Grpc.
proto_emit_service :: (_s: *proto_Schema, b: *String_Builder, service: *proto_Service) {
    proto_emit(b, "% :: struct {\n", proto_jai_name(service.name));

    for method: service.methods {
        name := method.jai;

        proto_emit(b, "% :: proto_g.Method.{ path = \"/%/%\", client_streaming = %, server_streaming = % };\n", name, service.full, method.name, ifx method.client_streaming then "true" else "false", ifx method.server_streaming then "true" else "false");

        for client: bool.[true, false] {
            suffix := ifx client then "Client" else "Server";
            input := ifx client then method.input.jai else method.output.jai;
            output := ifx client then method.output.jai else method.input.jai;
            pointer := ifx client then "" else "*";
            call_arg := ifx client then "*self.call" else "self.call";

            proto_emit(b, "%_% :: struct { call: %proto_g.Call;\n", name, suffix, pointer);
            proto_emit(b, "send :: (self: *%_%, value: %, options := proto_g.Send_Options.{}) -> proto_g.Status { return proto_g.send(%, value, options); }\n", name, suffix, input, call_arg);
            proto_emit(b, "receive :: (self: *%_%) -> %, proto_g.Receive_Kind { return proto_g.receive(%, %); }\n", name, suffix, output, call_arg, output);
            proto_emit(b, "cancelled :: (self: *%_%) -> bool { return proto_g.cancelled(%); }\n", name, suffix, call_arg);
            proto_emit(b, "deadline :: (self: *%_%) -> s64 { return proto_g.call_deadline(%); }\n", name, suffix, call_arg);
            proto_emit(b, "metadata :: (self: *%_%) -> [] proto_g.Metadata_Entry { return proto_g.initial_metadata(%); }\n", name, suffix, call_arg);

            proto_emit(b, "received_compressed :: (self: *%_%) -> bool { return proto_g.received_compressed(%); }\n", name, suffix, call_arg);

            if !client proto_emit(b, "peer_address :: (self: *%_%) -> proto_i.Address { return proto_g.peer_address(%); }\n", name, suffix, call_arg);

            if !client proto_emit(b, "set_initial_metadata :: (self: *%_%, entries: [] proto_g.Metadata_Entry) -> proto_g.Status { return proto_g.set_initial_metadata(%, entries); }\n", name, suffix, call_arg);

            if client {
                proto_emit(b, "close_send :: (self: *%_%) -> proto_g.Status { return proto_g.close_send(%); }\n", name, suffix, call_arg);
                proto_emit(b, "finish :: (self: *%_%) -> proto_g.Status { return proto_g.finish(%); }\n", name, suffix, call_arg);
                proto_emit(b, "cancel :: (self: *%_%) { proto_g.cancel(%); }\n", name, suffix, call_arg);
                proto_emit(b, "close :: (self: *%_%) { proto_g.close(%); }\n", name, suffix, call_arg);
            }

            append(b, "}\n");
        }

        proto_emit(b, "%_open :: (channel: *proto_g.Channel, options := proto_g.Call_Options.{}) -> %_Client, proto_g.Status { call, status := proto_g.open_call(channel, %, options); return .{ call }, status; }\n", name, name, name);

        if !method.client_streaming && !method.server_streaming {
            proto_emit(b, "%_call :: (channel: *proto_g.Channel, request: %, options := proto_g.Call_Options.{}) -> %, proto_g.Status {\n", name, method.input.jai, method.output.jai);
            proto_emit(b, "client, status := %_open(channel, options); if status.code != .OK return .{}, status; defer %_Client.close(*client);\n", name, name);
            proto_emit(b, "status = %_Client.send(*client, request); proto_g.free_status(*status);\n", name);
            proto_emit(b, "status = %_Client.close_send(*client); proto_g.free_status(*status);\n", name);
            proto_emit(b, "response, kind := %_Client.receive(*client); final := %_Client.finish(*client);\n", name, name);
            append(b, "if final.code != .OK || kind != .MESSAGE { proto_r.free(*response); return .{}, final; } return response, final; }\n");
        }
    }

    append(b, "Handlers :: struct { state: *void; options: proto_g.Call_Options;\n");
    for method: service.methods proto_emit(b, "%: #type (call: *%_Server, state: *void) -> proto_g.Status;\n", method.jai, method.jai);

    append(b, "}\n");

    for method: service.methods {
        name := method.jai;
        proto_emit(b, "%_serve :: (request: *proto_h.Request, data: *void) -> proto_h.Response {\n", name);
        proto_emit(b, "invoke :: (raw: *proto_g.Call, data: *void) -> proto_g.Status { handlers := cast(*Handlers) data; if !handlers.% return .{ code = .UNIMPLEMENTED }; typed := %_Server.{ raw }; return handlers.%(*typed, handlers.state); }\n", name, name, name);
        proto_emit(b, "handlers := cast(*Handlers) data; return proto_g.serve_call(request, %, invoke, data, handlers.options); }\n", name);
    }

    append(b, "register :: (app: *proto_h.App, handlers: *Handlers) {\n");
    for method: service.methods proto_emit(b, "proto_h.register_handler(app, .POST, \"/%/%\", %_serve, handlers);\n", service.full, method.name, method.jai);

    append(b, "}\n}\n");
}
