#scope_module

proto_emit_schema :: (s: *proto_Schema, b: *String_Builder) {
    append(b, "// Generated from proto3 by Extensions/Protobuf/Schema.\n");
    append(b, "#scope_module\nproto_basic_module :: #import \"Basic\";\nproto_runtime :: #import \"Extensions/Protobuf\";\nproto_variant_module :: #import \"Extensions/Variant\";\n");
    if s.options.map == .HASH_MAP append(b, "proto_map_module :: #import \"Extensions/Hash_Map\";\n");
    if s.options.map == .TABLE append(b, "proto_map_module :: #import \"Hash_Table\";\n");
    if s.options.map == .CUSTOM proto_emit(b, "proto_map_module :: #import \"%\";\n", s.options.map_module);

    append(b, "#scope_export\n");
    parts := split(s.package, ".");
    if s.package.count for parts proto_emit(b, "// Package namespace.\n% :: struct {\n", proto_jai_name(it));
    for s.roots proto_emit_node(s, b, it);
    if s.package.count for parts append(b, "}\n");
}

proto_map_type :: (s: *proto_Schema, f: *proto_Field) -> string {
    k := proto_field_type(f.map_entry.fields[0]);
    v := proto_field_type(f.map_entry.fields[1]);

    if s.options.map == .ARRAY return proto_schema_format("[..] proto_runtime.Map_Entry(%, %)", k, v);
    if s.options.map == .HASH_MAP return proto_schema_format("proto_map_module.Map(%, %)", k, v);
    if s.options.map == .TABLE return proto_schema_format("proto_map_module.Table(%, %)", k, v);

    return proto_schema_format("proto_map_module.%(%, %)", s.options.map_type, k, v);
}

proto_emit_node :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    if node.kind == .ENUM {
        proto_emit(b, "// Proto3 open enum.\n% :: enum s32 {\n", proto_jai_name(node.name));
        for node.values proto_emit(b, "// Schema value.\n% :: %;\n", it.name, it.value);
        append(b, "}\n");
        return;
    }

    proto_emit(b, "// Proto3 message.\n% :: struct {\n", proto_jai_name(node.name));
    for node.children proto_emit_node(s, b, it);

    for node.fields {
        if it.oneof.count continue;

        proto_emit(b, "// Schema field %.\n%: ", it.number, it.jai);

        if it.map_entry append(b, proto_map_type(s, it));
        else {
            if it.repeated append(b, "[..] ");
            if it.optional && !proto_is_message(it) proto_emit(b, "proto_runtime.Optional(%)", proto_field_type(it));
            else append(b, proto_field_type(it));
        }

        append(b, ";");
        proto_emit_field_notes(b, it);

        append(b, "\n");
    }

    for name: node.oneofs {
        proto_emit(b, "// Schema oneof.\n%: proto_variant_module.Variant(struct {\n", proto_jai_name(name));

        for node.fields if it.oneof == name {
            proto_emit(b, "%: %;", it.jai, proto_field_type(it));
            proto_emit_field_notes(b, it);
            append(b, "\n");
        }

        append(b, "});\n");
        proto_emit(b, "// Variant tag to protobuf field number (zero means unset).\nproto_%_numbers :: s64.[0", name);
        for node.fields if it.oneof == name proto_emit(b, ", %", it.number);

        append(b, "];\n");
    }

    append(b, "// Preserved unknown wire fields.\n_unknown: [..] u8;\n");
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

proto_emit_presence :: (b: *String_Builder, f: *proto_Field) {
    expr := proto_field_expr(f);
    if f.oneof.count proto_emit(b, "if value.%.which == .% {\n", proto_jai_name(f.oneof), proto_oneof_tag(f.jai));
    else if f.optional && !proto_is_message(f) proto_emit(b, "if value.%.has {\n", f.jai);
    else if proto_is_message(f) proto_emit(b, "if % != null {\n", expr);
    else if f.scalar == "string" || f.scalar == "bytes" proto_emit(b, "if %.count != 0 {\n", expr);
    else if f.scalar == "bool" proto_emit(b, "if % {\n", expr);
    else proto_emit(b, "if % != 0 {\n", expr);
}

proto_emit_item_size :: (b: *String_Builder, f: *proto_Field, input: string, proto_number: s64 = -1) {
    expr := input;
    if f.repeated && proto_is_message(f) expr = proto_schema_format("*%", expr);

    field_number := f.number;
    if proto_number >= 0 field_number = proto_number;

    tag := field_number * 8 + proto_field_wire(f);

    proto_emit(b, "n += proto_runtime.varint_size(%);\n", tag);

    if proto_is_message(f) {
        proto_emit(b, "length := %.proto_size(%, cache);\nn += proto_runtime.varint_size(cast(u64) length) + length;\n", f.target.jai, expr);
    } else proto_emit(b, "n += proto_runtime.scalar_size(%, .%);\n", expr, proto_field_kind(f));
}

proto_emit_item_write :: (b: *String_Builder, f: *proto_Field, input: string) {
    expr := input;
    if f.repeated && proto_is_message(f) expr = proto_schema_format("*%", expr);

    proto_emit(b, "proto_runtime.write_varint(w, %);\n", f.number * 8 + proto_field_wire(f));

    if proto_is_message(f) {
        proto_emit(b, "proto_runtime.write_varint(w, w.cache[w.next]);\nproto_runtime.write_message(w, %);\n", expr);
    } else proto_emit(b, "proto_runtime.write_scalar(w, %, .%);\n", expr, proto_field_kind(f));
}

proto_emit_map_entry :: (s: *proto_Schema, b: *String_Builder, f: *proto_Field, base: string) {
    if s.options.map == .ARRAY {
        proto_emit(b, "for item: %.% {\nentry := %.{key = item.key, value = item.value};\n", base, f.jai, f.map_entry.jai);
    } else proto_emit(b, "for item, key: %.% {\nentry := %.{key = key, value = item};\n", base, f.jai, f.map_entry.jai);
}

proto_emit_size :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    proto_emit(b, "// Internal size pass and preorder cache.\nproto_size :: (value: *%, cache: *[..] u32) -> s64 {\nslot := cache.count;\nproto_basic_module.array_add(cache, 0);\nn: s64;\n", node.jai);

    for f: node.fields {
        if f.map_entry {
            proto_emit_map_entry(s, b, f, "value");
            proto_emit(b, "length := %.proto_size(*entry, cache);\nn += proto_runtime.varint_size(%) + proto_runtime.varint_size(cast(u64) length) + length;\n}\n", f.map_entry.jai, f.number * 8 + 2);
        } else if f.repeated {
            if proto_packable(f) && f.packed {
                proto_emit(b, "if value.%.count {\npacked: s64;\nfor item: value.% packed += proto_runtime.scalar_size(item, .%);\nn += proto_runtime.varint_size(%) + proto_runtime.varint_size(cast(u64) packed) + packed;\n}\n", f.jai, f.jai, proto_field_kind(f), f.number * 8 + 2);
            } else {
                proto_emit(b, "for item: value.% {\n", f.jai);
                proto_emit_item_size(b, f, "item");
                append(b, "}\n");
            }
        } else {
            proto_emit_presence(b, f);
            if proto_is_message(f) && f.oneof.count proto_emit(b, "if % != null {\n", proto_field_expr(f));

            proto_emit_item_size(b, f, proto_field_expr(f));
            if proto_is_message(f) && f.oneof.count append(b, "}\n");

            append(b, "}\n");
        }
    }

    append(b, "n += value._unknown.count;\nproto_basic_module.assert(n <= 2147483647);\ncache.data[slot] = cast(u32) n;\nreturn n;\n}\n");
}

proto_emit_encode :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    proto_emit(b, "// Internal exact-size write pass.\nproto_encode :: (value: *%, w: *proto_runtime.Writer) {\n", node.jai);

    for f: node.fields {
        if f.map_entry {
            proto_emit_map_entry(s, b, f, "value");
            proto_emit(b, "proto_runtime.write_varint(w, %);\nproto_runtime.write_varint(w, w.cache[w.next]);\nproto_runtime.write_message(w, *entry);\n}\n", f.number * 8 + 2);
        } else if f.repeated {
            if proto_packable(f) && f.packed {
                proto_emit(b, "if value.%.count {\npacked: s64;\nfor item: value.% packed += proto_runtime.scalar_size(item, .%);\nproto_runtime.write_varint(w, %);\nproto_runtime.write_varint(w, cast(u64) packed);\nfor item: value.% proto_runtime.write_scalar(w, item, .%);\n}\n", f.jai, f.jai, proto_field_kind(f), f.number * 8 + 2, f.jai, proto_field_kind(f));
            } else {
                proto_emit(b, "for item: value.% {\n", f.jai);
                proto_emit_item_write(b, f, "item");
                append(b, "}\n");
            }
        } else {
            proto_emit_presence(b, f);
            if proto_is_message(f) && f.oneof.count proto_emit(b, "if % != null {\n", proto_field_expr(f));

            proto_emit_item_write(b, f, proto_field_expr(f));
            if proto_is_message(f) && f.oneof.count append(b, "}\n");

            append(b, "}\n");
        }
    }

    append(b, "proto_runtime.write_bytes(w, value._unknown);\n}\n");
}

proto_emit_item_free :: (b: *String_Builder, f: *proto_Field, expr: string) {
    if proto_is_message(f) proto_emit(b, "if % {\n%.proto_free(%);\nproto_basic_module.free(%);\n% = null;\n}\n", expr, f.target.jai, expr, expr, expr);
    else if f.scalar == "string" || f.scalar == "bytes" proto_emit(b, "proto_runtime.free_scalar(*%, .%);\n", expr, proto_field_kind(f));
}

proto_emit_free :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    for name: node.oneofs {
        proto_emit(b, "// Releases the active oneof member before switching.\nproto_clear_% :: (value: *%) {\n", name, node.jai);

        for f: node.fields if f.oneof == name {
            proto_emit(b, "if value.%.which == .% {\n", proto_jai_name(name), proto_oneof_tag(f.jai));
            proto_emit_item_free(b, f, proto_field_expr(f));
            append(b, "}\n");
        }

        proto_emit(b, "value.% = .{};\n}\n", proto_jai_name(name));
    }

    proto_emit(b, "// Releases all owned message allocations and resets fields.\nproto_free :: (value: *%) {\n", node.jai);
    for name: node.oneofs proto_emit(b, "proto_clear_%(value);\n", name);

    for f: node.fields {
        if f.oneof.count continue;

        if f.map_entry {
            proto_emit_map_entry(s, b, f, "value");
            proto_emit(b, "%.proto_free(*entry);\n}\n", f.map_entry.jai);
            if s.options.map == .ARRAY proto_emit(b, "proto_basic_module.array_free(value.%);\n", f.jai);
            else if s.options.map == .HASH_MAP proto_emit(b, "proto_map_module.deinit(*value.%);\n", f.jai);
            else if s.options.map == .TABLE proto_emit(b, "proto_map_module.table_reset(*value.%);\n", f.jai);
            else proto_emit(b, "proto_map_module.%(*value.%);\n", s.options.map_deinit, f.jai);
        } else if f.repeated {
            proto_emit(b, "for *item: value.% {\n", f.jai);
            if proto_is_message(f) proto_emit(b, "%.proto_free(item);\n", f.target.jai);
            else proto_emit_item_free(b, f, "item.*");
            proto_emit(b, "}\nproto_basic_module.array_free(value.%);\n", f.jai);
        } else {
            if f.optional && !proto_is_message(f) proto_emit(b, "if value.%.has {\n", f.jai);
            proto_emit_item_free(b, f, proto_field_expr(f));
            if f.optional && !proto_is_message(f) append(b, "}\n");
        }
    }

    append(b, "proto_basic_module.array_free(value._unknown);\nvalue.* = .{};\n}\n");
}

proto_emit_item_equal :: (b: *String_Builder, f: *proto_Field, a: string, c: string) {
    if proto_is_message(f) && f.repeated {
        proto_emit(b, "if !%.proto_equal(*%, *%) return false;\n", f.target.jai, a, c);
    } else if proto_is_message(f) {
        proto_emit(b, "if (% == null) != (% == null) return false;\nif % && !%.proto_equal(%, %) return false;\n", a, c, a, f.target.jai, a, c);
    } else if f.scalar == "bytes" proto_emit(b, "if !proto_runtime.bytes_equal(%, %) return false;\n", a, c);
    else proto_emit(b, "if % != % return false;\n", a, c);
}

proto_emit_equal :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node) {
    proto_emit(b, "// Deep equality including unknown wire bytes.\nproto_equal :: (a: *%, b: *%) -> bool {\n", node.jai, node.jai);
    for name: node.oneofs proto_emit(b, "if a.%.which != b.%.which return false;\n", proto_jai_name(name), proto_jai_name(name));

    for f: node.fields {
        if f.map_entry {
            proto_emit(b, "if a.%.count != b.%.count return false;\n", f.jai, f.jai);

            if s.options.map == .ARRAY {
                proto_emit(b, "for item: a.% {\nother := b.%[it_index];\n", f.jai, f.jai);
                proto_emit_item_equal(b, f.map_entry.fields[0], "item.key", "other.key");
                proto_emit_item_equal(b, f.map_entry.fields[1], "item.value", "other.value");
                append(b, "}\n");
            } else {
                proto_emit(b, "for item, key: a.% {\nfound := false;\nfor other, other_key: b.% {\nif key == other_key {\nfound = true;\n", f.jai, f.jai);
                proto_emit_item_equal(b, f.map_entry.fields[1], "item", "other");
                append(b, "break;\n}\n}\nif !found return false;\n}\n");
            }
        } else if f.repeated {
            proto_emit(b, "if a.%.count != b.%.count return false;\nfor item: a.% {\n", f.jai, f.jai, f.jai);
            proto_emit_item_equal(b, f, "item", proto_schema_format("b.%[it_index]", f.jai));
            append(b, "}\n");
        } else {
            if f.oneof.count proto_emit(b, "if a.%.which == .% {\n", proto_jai_name(f.oneof), proto_oneof_tag(f.jai));
            if f.optional && !proto_is_message(f) proto_emit(b, "if a.%.has != b.%.has return false;\nif a.%.has {\n", f.jai, f.jai, f.jai);
            proto_emit_item_equal(b, f, proto_field_expr(f, "a"), proto_field_expr(f, "b"));
            if f.oneof.count || (f.optional && !proto_is_message(f)) append(b, "}\n");
        }
    }

    append(b, "return proto_runtime.bytes_equal(a._unknown, b._unknown);\n}\n");
}

proto_emit_read_value :: (b: *String_Builder, f: *proto_Field, expr: string, path: string) {
    if proto_is_message(f) {
        proto_emit(b, "if !% % = proto_basic_module.New(%);\nif !proto_runtime.read_message(r, %) return proto_runtime.field_error(r, \"%\");\n", expr, expr, f.target.jai, expr, path);
    } else proto_emit(b, "if !proto_runtime.read_scalar(r, *%, .%) return proto_runtime.field_error(r, \"%\");\n", expr, proto_field_kind(f), path);
}

proto_emit_read_field :: (s: *proto_Schema, b: *String_Builder, node: *proto_Node, f: *proto_Field) {
    proto_emit(b, "// Reads one occurrence of field %.\nproto_read_% :: (value: *%, r: *proto_runtime.Reader, wire: u64) -> bool {\n", f.number, f.number, node.jai);

    if f.repeated && proto_packable(f) {
        proto_emit(b, "if wire == 2 {\nlength := proto_runtime.read_length(r);\nif r.error.kind != .NONE return proto_runtime.field_error(r, \"%\");\nold_end := r.end;\nr.end = r.pos + length;\ndefer r.end = old_end;\nwhile proto_runtime.remaining(r) > 0 {\nitem: %;\nif !proto_runtime.read_scalar(r, *item, .%) return proto_runtime.field_error(r, \"%\", value.%.count);\nproto_basic_module.array_add(*value.%, item);\n}\nreturn true;\n}\n", f.name, proto_field_type(f), proto_field_kind(f), f.name, f.jai, f.jai);
    }

    proto_emit(b, "if wire != % return false;\n", proto_field_wire(f));

    if f.map_entry {
        proto_emit(b, "entry: %;\ndefer %.proto_free(*entry);\nif !proto_runtime.read_message(r, *entry) return proto_runtime.field_error(r, \"%\");\n", f.map_entry.jai, f.map_entry.jai, f.name);

        if s.options.map == .ARRAY {
            proto_emit(b, "proto_basic_module.array_add(*value.%, .{entry.key, entry.value});\nentry.key = .{}; entry.value = .{};\n", f.jai);
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
        if proto_is_message(f) append(b, "ok = proto_runtime.read_message(r, *item);\n");
        else proto_emit(b, "ok = proto_runtime.read_scalar(r, *item, .%);\n", proto_field_kind(f));

        append(b, "if !ok {\n");
        if proto_is_message(f) proto_emit(b, "%.proto_free(*item);\n", f.target.jai);
        else proto_emit_item_free(b, f, "item");

        proto_emit(b, "return proto_runtime.field_error(r, \"%\", value.%.count);\n}\nproto_basic_module.array_add(*value.%, item);\n", f.name, f.jai, f.jai);
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
    for node.fields proto_emit_read_field(s, b, node, it);

    proto_emit(b, "// Generated bounded wire loop; common tags precede the field-number fallback.\nproto_merge :: (value: *%, r: *proto_runtime.Reader) -> bool {\nwhile proto_runtime.remaining(r) > 0 {\nstart := r.pos;\ntag := proto_runtime.read_tag(r);\nif r.error.kind != .NONE return false;\n", node.jai);

    for f: node.fields {
        tag := f.number * 8 + proto_field_wire(f);
        if f.repeated && proto_packable(f) && f.packed tag = f.number * 8 + 2;
        proto_emit(b, "if tag == % {\nif !proto_read_%(value, r, tag & 7) return false;\ncontinue;\n}\n", tag, f.number);
    }

    append(b, "handled := false;\nif tag >> 3 == {\n");
    for f: node.fields proto_emit(b, "case %; handled = proto_read_%(value, r, tag & 7);\n", f.number, f.number);

    append(b, "}\nif r.error.kind != .NONE return false;\nif handled continue;\nif !proto_runtime.skip(r, tag) return false;\nif !r.options.discard_unknown {\nraw: [] u8 = .{r.pos - start, r.bytes.data + start};\nproto_basic_module.array_add(*value._unknown, ..raw);\n}\n}\nreturn true;\n}\n");
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
    if s.options.map == .HASH_MAP proto_emit(b, "proto_map_module.insert(*value.%, entry.key, entry.value);\n", f.jai);
    else if s.options.map == .TABLE proto_emit(b, "proto_map_module.table_set(*value.%, entry.key, entry.value);\n", f.jai);
    else proto_emit(b, "proto_map_module.%(*value.%, entry.key, entry.value);\n", s.options.map_insert, f.jai);
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
