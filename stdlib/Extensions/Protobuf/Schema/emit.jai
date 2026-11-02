#scope_module

proto_emit_schema :: (s: *proto_Schema, b: *String_Builder) {
    append(b, "// Generated from proto3 by Extensions/Protobuf/Schema.\n");
    append(b, "#scope_module\nproto_b :: #import \"Basic\";\nproto_r :: #import \"Extensions/Protobuf\";\nproto_v :: #import \"Extensions/Variant\";\n");
    if s.options.map == .HASH_MAP append(b, "proto_m :: #import \"Extensions/Hash_Map\";\n");
    if s.options.map == .TABLE append(b, "proto_m :: #import \"Hash_Table\";\n");
    if s.options.map == .CUSTOM proto_emit(b, "proto_m :: #import \"%\";\n", s.options.map_module);

    append(b, "#scope_export\n");
    proto_emit_namespace(s, b, "");
}

proto_emit_namespace :: (s: *proto_Schema, b: *String_Builder, prefix: string) {
    namespaces: [..] string;

    for root: s.roots {
        if root.package == prefix {
            proto_emit_node(s, b, root);
            continue;
        }

        rest := root.package;

        if prefix.count {
            head := proto_schema_format("%.", prefix);
            if !begins_with(rest, head) continue;
            rest = slice(rest, head.count, rest.count - head.count);
        }

        if !rest.count continue;

        part := rest;

        for rest if it == #char "." {
            part = slice(rest, 0, it_index);
            break;
        }

        seen := false;
        for namespaces if it == part seen = true;
        if seen continue;

        array_add(*namespaces, part);
        proto_emit(b, "% :: struct {\n", proto_jai_name(part));
        proto_emit_namespace(s, b, proto_join_name(prefix, part));
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

        append(b, ";");
        proto_emit_field_notes(b, it);

        append(b, "\n");
    }

    for name: node.oneofs {
        proto_emit(b, "%: proto_v.Variant(struct {\n", proto_jai_name(name));

        for node.fields if it.oneof == name {
            proto_emit(b, "%: %;", it.jai, proto_field_type(it));
            proto_emit_field_notes(b, it);
            append(b, "\n");
        }

        append(b, "});\n");
        proto_emit(b, "proto_%_numbers :: s64.[0", name);
        for node.fields if it.oneof == name proto_emit(b, ", %", it.number);

        append(b, "];\n");
    }

    append(b, "_unknown: [..] u8;\n");
    proto_emit_size(s, b, node);
    proto_emit_encode(s, b, node);
    proto_emit_free(s, b, node);
    proto_emit_equal(s, b, node);
    proto_emit_merge(s, b, node);
    append(b, "}\n");
}

proto_field_expr :: (f: *proto_Field, base: string = "value") -> string {
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
    proto_emit(b, "proto_size :: (value: *%, cache: *[..] u32) -> s64 {\nslot := cache.count;\nproto_b.array_add(cache, 0);\nn: s64;\n", node.jai);

    for f: node.fields {
        if f.map_entry && s.options.map != .CUSTOM {
            proto_emit(b, "n += proto_r.size_map(*value.%, %, .%, %, cache);\n", f.jai, proto_entry_type(f), proto_map_mode(s), f.number * 8 + 2);
        } else if f.map_entry {
            proto_emit_map_entry(s, b, f, "value");
            proto_emit(b, "n += proto_r.size_field(*entry, .MESSAGE, %, cache);\n}\n", f.number * 8 + 2);
        } else proto_emit_field_call(b, f, false);
    }

    append(b, "n = proto_b.min(n + value._unknown.count, 2147483648);\ncache.data[slot] = cast(u32) n;\nreturn n;\n}\n");
}

proto_emit_encode :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    proto_emit(b, "proto_encode :: (value: *%, w: *proto_r.Writer) {\n", node.jai);

    for f: node.fields {
        if f.map_entry && s.options.map != .CUSTOM {
            proto_emit(b, "proto_r.encode_map(w, *value.%, %, .%, %);\n", f.jai, proto_entry_type(f), proto_map_mode(s), f.number * 8 + 2);
        } else if f.map_entry {
            proto_emit_map_entry(s, b, f, "value");
            proto_emit(b, "proto_r.encode_field(w, *entry, .MESSAGE, %);\n}\n", f.number * 8 + 2);
        } else proto_emit_field_call(b, f, true);
    }

    append(b, "proto_r.write_bytes(w, value._unknown);\n}\n");
}

proto_emit_field_call :: (b: *String_Builder, f: *proto_Field, writing: bool) {
    prefix := "n += proto_r.size_";
    suffix := ", cache";

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
        argument = proto_schema_format("value.%", f.jai);
        if writing argument = proto_schema_format("w, %", argument);
    }

    if f.oneof.count {
        proto_emit(b, "if value.%.which == .% ", proto_jai_name(f.oneof), proto_oneof_tag(f.jai));
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
    for name: node.oneofs {
        proto_emit(b, "proto_clear_% :: (value: *%) {\n", name, node.jai);

        for f: node.fields if f.oneof == name && (proto_is_message(f) || f.scalar == "string" || f.scalar == "bytes") {
            proto_emit(b, "if value.%.which == .% {\n", proto_jai_name(name), proto_oneof_tag(f.jai));
            proto_emit_item_free(b, f, proto_field_expr(f));
            append(b, "}\n");
        }

        proto_emit(b, "value.% = .{};\n}\n", proto_jai_name(name));
    }

    proto_emit(b, "proto_free :: (value: *%) {\n", node.jai);
    for name: node.oneofs proto_emit(b, "proto_clear_%(value);\n", name);

    for f: node.fields {
        if f.oneof.count continue;

        if f.map_entry && s.options.map != .CUSTOM {
            proto_emit(b, "proto_r.free_map(*value.%, %, .%);\n", f.jai, proto_entry_type(f), proto_map_mode(s));
        } else if f.map_entry {
            proto_emit_map_entry(s, b, f, "value");
            proto_emit(b, "%.proto_free(*entry);\n}\n", proto_entry_type(f));
            if s.options.map == .ARRAY proto_emit(b, "proto_b.array_free(value.%);\n", f.jai);
            else if s.options.map == .HASH_MAP proto_emit(b, "proto_m.deinit(*value.%);\n", f.jai);
            else if s.options.map == .TABLE proto_emit(b, "proto_m.table_reset(*value.%);\n", f.jai);
            else proto_emit(b, "proto_m.%(*value.%);\n", s.options.map_deinit, f.jai);
        } else if f.repeated {
            proto_emit(b, "proto_r.free_repeated(*value.%, .%);\n", f.jai, proto_field_kind(f));
        } else if proto_is_message(f) || f.scalar == "string" || f.scalar == "bytes" {
            if f.optional && !proto_is_message(f) proto_emit(b, "if value.%.has {\n", f.jai);
            proto_emit_item_free(b, f, proto_field_expr(f));
            if f.optional && !proto_is_message(f) append(b, "}\n");
        }
    }

    append(b, "proto_b.array_free(value._unknown);\nvalue.* = .{};\n}\n");
}

proto_emit_item_equal :: (b: *String_Builder, f: *proto_Field, a: string, c: string) {
    proto_emit(b, "if !proto_r.equal_field(%, %, .%) return false;\n", a, c, proto_field_kind(f));
}

proto_emit_equal :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    proto_emit(b, "proto_equal :: (a: *%, b: *%) -> bool {\n", node.jai, node.jai);
    for name: node.oneofs proto_emit(b, "if a.%.which != b.%.which return false;\n", proto_jai_name(name), proto_jai_name(name));

    for f: node.fields {
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
        proto_emit(b, "if !% % = proto_b.New(%);\nif !proto_r.read_message(r, %) return proto_r.field_error(r, \"%\");\n", expr, expr, f.target.jai, expr, path);
    } else proto_emit(b, "if !proto_r.read_scalar(r, *%, .%) return proto_r.field_error(r, \"%\");\n", expr, proto_field_kind(f), path);
}

proto_emit_read_field :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node, f: *proto_Field) {
    proto_emit(b, "#if field == % {\n", f.number);

    if !f.oneof.count && (!f.map_entry || s.options.map != .CUSTOM) {
        mode := "field";
        if f.repeated mode = "repeated";
        if f.optional && !proto_is_message(f) mode = "optional";
        if f.map_entry proto_emit(b, "return proto_r.read_map(r, *value.%, %, .%, wire, \"%\");\n}\n", f.jai, proto_entry_type(f), proto_map_mode(s), f.name);
        else proto_emit(b, "return proto_r.read_%(r, *value.%, .%, wire, \"%\");\n}\n", mode, f.jai, proto_field_kind(f), f.name);

        return;
    }

    proto_emit(b, "if wire != % return false;\n", proto_field_wire(f));

    if f.map_entry {
        proto_emit(b, "entry: %;\ndefer %.proto_free(*entry);\nif !proto_r.read_message(r, *entry) return proto_r.field_error(r, \"%\");\n", proto_entry_type(f), proto_entry_type(f), f.name);

        if s.options.map == .ARRAY {
            proto_emit(b, "proto_b.array_add(*value.%, .{entry.key, entry.value});\nentry.key = .{}; entry.value = .{};\n", f.jai);
        } else {
            proto_emit(b, "for old, key: value.% {\nif key == entry.key {\nprevious := old;\n", f.jai);
            proto_emit_item_free(b, f.map_entry.fields[1], "previous");
            proto_emit_item_free(b, f.map_entry.fields[0], "entry.key");
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

        proto_emit(b, "return proto_r.field_error(r, \"%\", value.%.count);\n}\nproto_b.array_add(*value.%, item);\n", f.name, f.jai, f.jai);
    } else {
        if f.oneof.count {
            proto_emit(b, "if value.%.which != .% {\nproto_clear_%(value);\nvalue.%.which = .%;\n}\n", proto_jai_name(f.oneof), proto_oneof_tag(f.jai), f.oneof, proto_jai_name(f.oneof), proto_oneof_tag(f.jai));
        }

        proto_emit_read_value(b, f, proto_field_expr(f), f.name);
        if f.optional && !proto_is_message(f) proto_emit(b, "value.%.has = true;\n", f.jai);
    }

    append(b, "return true;\n}\n");
}

proto_emit_merge :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    if node.fields.count {
        proto_emit(b, "proto_read :: inline (value: *%, r: *proto_r.Reader, $field: s64, wire: u64) -> bool {\n", node.jai);
        for node.fields proto_emit_read_field(s, b, node, it);
        append(b, "return false;\n}\n");
    }

    proto_emit(b, "proto_merge :: (value: *%, r: *proto_r.Reader) -> bool {\nwhile proto_r.remaining(r) > 0 {\nstart := r.pos;\ntag := proto_r.read_tag(r);\nif r.error.kind != .NONE return false;\n", node.jai);

    for f: node.fields {
        tag := f.number * 8 + proto_field_wire(f);
        if f.repeated && proto_packable(f) && f.packed tag = f.number * 8 + 2;
        proto_emit(b, "if tag == % { if !proto_read(value, r, %, tag & 7) return false; continue; }\n", tag, f.number);
    }

    append(b, "handled := false;\nif tag >> 3 == {\n");
    for f: node.fields proto_emit(b, "case %; handled = proto_read(value, r, %, tag & 7);\n", f.number, f.number);

    append(b, "}\nif r.error.kind != .NONE return false;\nif handled continue;\nif !proto_r.skip(r, tag) return false;\nif !r.options.discard_unknown {\nraw: [] u8 = .{r.pos - start, r.bytes.data + start};\nproto_b.array_add(*value._unknown, ..raw);\n}\n}\nreturn true;\n}\n");
}

// Specialize literal templates once, so generating a schema does not scan every output byte.
// Arguments remain a small string/s64-only formatter, rather than the reflective Basic printer.
proto_emit :: (b: *String_Builder, $template: string, args: ..Any) {
    #insert -> string {
        code: String_Builder;
        start: s64;
        argument: s64;

        for template {
            if it != #char "%" continue;
            print(*code, "append(b, %);\nproto_emit_argument(b, args[%]);\n", proto_quote(slice(template, start, it_index - start)), argument);
            argument += 1;
            start = it_index + 1;
        }

        print(*code, "append(b, %);\nassert(args.count == %);\n", proto_quote(slice(template, start, template.count - start)), argument);

        return builder_to_string(*code);
    };
}

proto_emit_argument :: (b: *String_Builder, arg: Any) {
    if arg.type.type == .STRING {
        append(b, (cast(*string) arg.value_pointer).*);
        return;
    }

    assert(arg.type.type == .INTEGER && arg.type.runtime_size == 8);
    value := (cast(*s64) arg.value_pointer).*;
    digits: [20] u8;
    end := digits.count;
    negative := value < 0;
    bits: u64;

    if negative bits = cast(u64) -value;
    else bits = cast(u64) value;

    while true {
        end -= 1;
        digits[end] = cast(u8) (bits % 10) + #char "0";
        bits /= 10;
        if !bits break;
    }

    if negative {
        end -= 1;
        digits[end] = #char "-";
    }

    append(b, string.{ digits.count - end, digits.data + end });
}

proto_schema_format :: ($template: string, args: ..Any) -> string {
    b: String_Builder;
    proto_emit(*b, template, ..args);
    return builder_to_string(*b);
}

proto_emit_map_insert :: (s: *proto_Schema, b: *String_Builder, f: *proto_Field) {
    if s.options.map == .HASH_MAP proto_emit(b, "proto_m.insert(*value.%, entry.key, entry.value);\n", f.jai);
    else if s.options.map == .TABLE proto_emit(b, "proto_m.table_set(*value.%, entry.key, entry.value);\n", f.jai);
    else proto_emit(b, "proto_m.%(*value.%, entry.key, entry.value);\n", s.options.map_insert, f.jai);
}

// A Jai string literal, or a JSON string inside a Json note. Both escape control bytes.
proto_quote :: (text: string, json := false) -> string {
    b: String_Builder;

    append(*b, "\"");

    for text {
        if it == 34 append(*b, "\\\"");
        else if it == 92 append(*b, "\\\\");
        else if it < 32 {
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

    if f.has_json_name && f.json_name != f.jai {
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
