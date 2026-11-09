// Specialized field operations shared by all generated message types.

// Writes a constant tag as literal bytes, with no runtime varint loop.
write_tag :: inline (w: *Writer, $tag: s64) {
    w.bytes[w.pos] = cast(u8) ((tag & 127) | (ifx tag >= 128 then 128 else 0));
    w.pos += 1;
    #if tag >= 128 write_tag(w, tag >> 7);
}

// Size of a tag, evaluated at generation/specialization time.
tag_size :: (tag: s64) -> s64 {
    n: s64 = 1;

    while tag >= 128 {
        n += 1;
        tag >>= 7;
    }

    return n;
}

// Tests implicit field presence; forced presence also encodes scalar defaults.
has_field :: inline (value: $T, $kind: Scalar, force := false) -> bool {
    #if (kind == .MESSAGE || kind == .GROUP) return value != null;
    else #if kind == .STRING || kind == .RAW_STRING || kind == .BYTES return force || value.count != 0;
    else #if kind == .BOOL return force || value;
    else #if kind == .FLOAT return force || (cast(*u32) *value).* != 0;
    else #if kind == .DOUBLE return force || (cast(*u64) *value).* != 0;
    else return force || value != 0;
}

// Sizes one singular field and appends nested message sizes in preorder.
size_field :: inline (value: $T, $kind: Scalar, $tag: s64, cache: *[..] u32, force := false) -> s64 {
    TAG_SIZE :: #run tag_size(tag);

    if !has_field(value, kind, force) return 0;

    #if (kind == .MESSAGE || kind == .GROUP) {
        n := size_message(value, cache);
        #if kind == .GROUP return TAG_SIZE * 2 + n;
        else return TAG_SIZE + varint_size(cast(u64) n) + n;
    } else return TAG_SIZE + scalar_size(value, kind);
}

// Saturates oversized message sizes before storing the u32 cache entry.
size_message :: (value: *$T, cache: *[..] u32) -> s64 {
    return T.proto_size(value, cache);
}

// Writes one singular field using the preorder size cache.
encode_field :: inline (w: *Writer, value: $T, $kind: Scalar, $tag: s64, force := false) {
    if !has_field(value, kind, force) return;

    write_tag(w, tag);

    #if (kind == .MESSAGE || kind == .GROUP) {
        #if kind != .GROUP write_varint(w, w.cache[w.next]);
        write_message(w, value);
        #if kind == .GROUP write_tag(w, tag + 1);
    } else write_scalar(w, value, kind);
}

// Sizes an explicitly present scalar.
size_optional :: inline (value: Optional($T), $kind: Scalar, $tag: s64, cache: *[..] u32) -> s64 {
    if !value.has return 0;
    return size_field(value.value, kind, tag, cache, true);
}

// Writes an explicitly present scalar.
encode_optional :: inline (w: *Writer, value: Optional($T), $kind: Scalar, $tag: s64) {
    if value.has encode_field(w, value.value, kind, tag, true);
}

// Sizes repeated fields, with packed scalars sharing one length prefix.
size_repeated :: inline (values: [] $T, $kind: Scalar, $tag: s64, $packed: bool, cache: *[..] u32) -> s64 {
    TAG_SIZE :: #run tag_size(tag);
    n: s64;

    #if packed {
        if !values.count return 0;

        #if kind == .FIXED32 || kind == .SFIXED32 || kind == .FLOAT n = values.count * 4;
        else #if kind == .FIXED64 || kind == .SFIXED64 || kind == .DOUBLE n = values.count * 8;
        else for values n += scalar_size(it, kind);

        return TAG_SIZE + varint_size(cast(u64) n) + n;
    } else {
        for values {
            #if (kind == .MESSAGE || kind == .GROUP) n += size_field(*it, kind, tag, cache, true);
            else n += size_field(it, kind, tag, cache, true);
        }

        return n;
    }
}

// Writes repeated fields, including packed scalars.
encode_repeated :: inline (w: *Writer, values: [] $T, $kind: Scalar, $tag: s64, $packed: bool) {
    #if packed {
        if !values.count return;

        n: s64;

        #if kind == .FIXED32 || kind == .SFIXED32 || kind == .FLOAT n = values.count * 4;
        else #if kind == .FIXED64 || kind == .SFIXED64 || kind == .DOUBLE n = values.count * 8;
        else for values n += scalar_size(it, kind);

        write_tag(w, tag);
        write_varint(w, cast(u64) n);

        // Every supported target (x64, arm64, wasm) is little endian.
        #if kind == .FIXED32 || kind == .SFIXED32 || kind == .FLOAT || kind == .FIXED64 || kind == .SFIXED64 || kind == .DOUBLE {
            write_bytes(w, .{ n, cast(*u8) values.data });
        } else for values write_scalar(w, it, kind);
    } else for values {
        #if (kind == .MESSAGE || kind == .GROUP) encode_field(w, *it, kind, tag, true);
        else encode_field(w, it, kind, tag, true);
    }
}

// Releases a scalar or optional message pointer.
free_field :: inline (value: *$T, $kind: Scalar) {
    #if (kind == .MESSAGE || kind == .GROUP) {
        if value.* {
            Message :: type_of(value.*.*);
            Message.proto_free(value.*);
            free_owned_pointer(value.*);
            value.* = null;
        }
    } else free_scalar(value, kind);
}

// Shared map-entry codec; it never stores unknown fields or emits synthetic schema types.
Entry_Codec :: struct(K: Type, V: Type, key_kind: Scalar, value_kind: Scalar) {
    key: K;
    value: V = #run entry_default(V);
    invalid_enum: bool;
    proto_storage: *Storage_Block;
    proto_has_required :: false;

    proto_check :: (entry: *Entry_Codec, r: *Reader) -> bool {
        #if value_kind == .MESSAGE return check_message(entry.value, r);
        else return true;
    }

    proto_size :: (entry: *Entry_Codec, cache: *[..] u32) -> s64 {
        slot := cache.count;

        array_add(cache, 0);
        n := size_field(entry.key, key_kind, 8, cache, true) + size_field(entry.value, value_kind, 16 + cast(s64) scalar_wire(value_kind), cache, true);

        cache.data[slot] = cast(u32) min(n, 2147483648);

        return min(n, 2147483648);
    }

    proto_encode :: (entry: *Entry_Codec, w: *Writer) {
        encode_field(w, entry.key, key_kind, 8 + cast(s64) scalar_wire(key_kind), true);
        encode_field(w, entry.value, value_kind, 16 + cast(s64) scalar_wire(value_kind), true);
    }

    proto_free :: (entry: *Entry_Codec) {
        free_field(*entry.key, key_kind);
        free_field(*entry.value, value_kind);
    }

    proto_merge :: (entry: *Entry_Codec, r: *Reader) -> bool {
        suppressed := r.suppress_enum_unknown;

        #if value_kind == .CLOSED_ENUM r.suppress_enum_unknown = true;
        defer r.suppress_enum_unknown = suppressed;

        while remaining(r) > 0 {
            tag := read_tag(r);

            if r.error.kind != .NONE return false;

            if tag == 8 + scalar_wire(key_kind) {
                if !read_scalar(r, *entry.key, key_kind) return field_error(r, "key");
            } else if tag == 16 + scalar_wire(value_kind) {
                #if value_kind == .MESSAGE {
                    if !entry.value entry.value = new_message(r, type_of(entry.value.*));
                    if !read_message(r, entry.value) return field_error(r, "value");
                } else {
                    if !read_scalar(r, *entry.value, value_kind) return field_error(r, "value");
                    #if value_kind == .CLOSED_ENUM if r.enum_rejected entry.invalid_enum = true;
                }
            } else if !skip(r, tag) return false;
        }

        return true;
    }
}

// Reads a singular scalar/message occurrence; wrong wire types remain unknown.
read_field :: inline (r: *Reader, value: *$T, $kind: Scalar, wire: u64, path: string) -> bool {
    if wire != scalar_wire(kind) return false;

    #if (kind == .MESSAGE || kind == .GROUP) {
        if !value.* value.* = new_message(r, type_of(value.*.*));
        #if kind == .GROUP ok := read_group(r, value.*);
        else ok := read_message(r, value.*);
        if !ok return field_error(r, path);
    } else if !read_scalar(r, value, kind) return field_error(r, path);

    return true;
}

// Reads an explicitly present scalar, setting presence only after success.
read_optional :: inline (r: *Reader, value: *Optional($T), $kind: Scalar, wire: u64, path: string) -> bool {
    // The absent payload may be a static schema default, so never free it.
    #if kind == .STRING || kind == .RAW_STRING || kind == .BYTES {
        item: T;
        if !read_field(r, *item, kind, wire, path) return false;
        if value.has release_scalar(r, *value.value, kind);
        value.value = item;
    } else if !read_field(r, *value.value, kind, wire, path) return false;

    #if kind == .CLOSED_ENUM {
        if !r.enum_rejected value.has = true;
    } else value.has = true;

    return true;
}

// Reads packed or expanded repeated occurrences, preserving bounds and index paths.
read_repeated :: (r: *Reader, values: *[..] $T, $kind: Scalar, wire: u64, path: string) -> bool {
    #if kind != .MESSAGE && kind != .GROUP && kind != .RAW_STRING && kind != .STRING && kind != .BYTES {
        if wire == 2 {
            n := read_length(r);

            if r.error.kind != .NONE return field_error(r, path);

            #if kind == .FIXED32 || kind == .SFIXED32 || kind == .FLOAT || kind == .FIXED64 || kind == .SFIXED64 || kind == .DOUBLE {
                WIDTH :: size_of(T);
                if n % WIDTH {
                    fail(r, .TRUNCATED);
                    return field_error(r, path, values.count + n / WIDTH);
                }

                old_count := values.count;

                array_resize(values, old_count + n / WIDTH, false);
                if n memcpy(values.data + old_count, r.bytes.data + r.pos, n);

                r.pos += n;

                return true;
            }

            old_end := r.end;

            r.end = r.pos + n;
            defer r.end = old_end;

            while remaining(r) > 0 {
                item: T;
                if !read_scalar(r, *item, kind) return field_error(r, path, values.count);

                #if kind == .CLOSED_ENUM {
                    if !r.enum_rejected append_decoded(r, values, item);
                } else append_decoded(r, values, item);
            }

            return true;
        }
    }

    if wire != scalar_wire(kind) return false;

    // A message is decoded straight into its new slot rather than copied in afterwards.
    // Nothing else appends to this array meanwhile, so the slot stays put.
    #if kind == .MESSAGE || kind == .GROUP {
        slot := add_decoded(r, values);

        #if kind == .GROUP decoded := read_group(r, slot);
        else decoded := read_message(r, slot);

        if !decoded {
            release_entry(r, slot);
            values.count -= 1;
            return field_error(r, path, values.count);
        }
    } else {
        item: T;

        if !read_scalar(r, *item, kind) {
            release_scalar(r, *item, kind);
            return field_error(r, path, values.count);
        }

        #if kind == .CLOSED_ENUM {
            if !r.enum_rejected append_decoded(r, values, item);
        } else append_decoded(r, values, item);
    }

    return true;
}

// Compares scalar values or optional message pointers by content.
equal_field :: inline (a: $T, b: T, $kind: Scalar) -> bool {
    #if (kind == .MESSAGE || kind == .GROUP) {
        if (a == null) != (b == null) return false;
        M :: type_of(a.*);
        return !a || M.proto_equal(a, b);
    } else #if kind == .BYTES return bytes_equal(a, b);
    else return a == b;
}

// Compares explicit presence, ignoring the payload when absent.
equal_optional :: inline (a: Optional($T), b: Optional(T), $kind: Scalar) -> bool {
    return a.has == b.has && (!a.has || equal_field(a.value, b.value, kind));
}

// Compares repeated field values in order.
equal_repeated :: (a: [] $T, b: [] T, $kind: Scalar) -> bool {
    if a.count != b.count return false;

    for a {
        #if (kind == .MESSAGE || kind == .GROUP) {
            if !T.proto_equal(*it, *b[it_index]) return false;
        } else if !equal_field(it, b[it_index], kind) return false;
    }

    return true;
}

// Releases every repeated element and its container.
free_repeated :: (values: *[..] $T, $kind: Scalar) {
    #if (kind == .MESSAGE || kind == .GROUP) for *values.* T.proto_free(it);
    else #if kind == .STRING || kind == .RAW_STRING || kind == .BYTES for *values.* free_scalar(it, kind);
    array_free(values.*);
}

// Map container selection for the shared map operations.
Map_Storage :: enum {
    HASH_MAP;
    TABLE;
    ARRAY;
}

// Sizes all map entries using one shared codec specialization.
size_map :: (map: *$T, $E: Type, $mode: Map_Storage, $tag: s64, cache: *[..] u32) -> s64 {
    n: s64;

    #if mode == .ARRAY for map.* {
        entry := E.{ key = it.key, value = it.value };
        n += size_field(*entry, .MESSAGE, tag, cache);
    } else for item, key: map.* {
        entry := E.{ key = key, value = item };
        n += size_field(*entry, .MESSAGE, tag, cache);
    }

    return n;
}

// Writes all map entries in the container's iteration order.
encode_map :: (w: *Writer, map: *$T, $E: Type, $mode: Map_Storage, $tag: s64) {
    #if mode == .ARRAY for map.* {
        entry := E.{ key = it.key, value = it.value };
        encode_field(w, *entry, .MESSAGE, tag);
    } else for item, key: map.* {
        entry := E.{ key = key, value = item };
        encode_field(w, *entry, .MESSAGE, tag);
    }
}

// Releases map keys, values and container storage.
free_map :: (map: *$T, $E: Type, $mode: Map_Storage) {
    #if mode == .ARRAY {
        for map.* {
            entry := E.{ key = it.key, value = it.value };
            E.proto_free(*entry);
        }

        array_free(map.*);
    } else {
        for item, key: map.* {
            entry := E.{ key = key, value = item };
            E.proto_free(*entry);
        }

        #if mode == .HASH_MAP proto_maps.deinit(map);
        else proto_tables.table_reset(map);
    }
}

// Map equality uses lookup for HASH_MAP and TABLE, or order for ARRAY.
equal_map :: (a: *$T, b: *T, $E: Type, $mode: Map_Storage) -> bool {
    if a.count != b.count return false;

    #if mode == .ARRAY for a.* {
        if !equal_field(it.key, b.*[it_index].key, E.key_kind) || !equal_field(it.value, b.*[it_index].value, E.value_kind) return false;
    } else for item, key: a.* {
        #if mode == .HASH_MAP p := proto_maps.find(b, key);
        else p := proto_tables.table_find_pointer(b, key);
        if !p || !equal_field(item, p.*, E.value_kind) return false;
    }

    return true;
}

// Reads a map entry, replacing owned duplicate keys and values after a successful read.
read_map :: (r: *Reader, map: *$T, $E: Type, $mode: Map_Storage, wire: u64, path: string) -> bool {
    if wire != 2 return false;

    entry: E;
    start := r.tag_start;

    defer release_entry(r, *entry);
    if !read_message(r, *entry) return field_error(r, path);

    if entry.invalid_enum {
        if !r.options.discard_unknown && r.unknown {
            raw: [] u8 = .{ r.pos - start, r.bytes.data + start };
            append_unknown(r, raw);
        }

        return true;
    }

    #if E.value_kind == .MESSAGE if !entry.value entry.value = new_message(r, type_of(entry.value.*));

    #if mode == .ARRAY array_add(map, .{ entry.key, entry.value });
    else {
        #if mode == .HASH_MAP p := proto_maps.find(map, entry.key);
        else p := proto_tables.table_find_pointer(map, entry.key);

        if p {
            release_field(r, p, E.value_kind);
            release_field(r, *entry.key, E.key_kind);

            p.* = entry.value;
            entry.value = .{};

            return true;
        }

        #if mode == .HASH_MAP proto_maps.insert(map, entry.key, entry.value);
        else proto_tables.table_set(map, entry.key, entry.value);
    }

    entry.key = .{};
    entry.value = .{};

    return true;
}

#scope_module
proto_maps :: #import "Extensions/Hash_Map";
proto_tables :: #import "Hash_Table";
#import "Extensions/Hash_Map";
#import "Hash_Table";

#scope_module

entry_default :: ($T: Type) -> T {
    #if type_info(T).type == .ENUM {
        #if (cast(*Type_Info_Enum) type_info(T)).values.count return cast(T) (cast(*Type_Info_Enum) type_info(T)).values[0];
    }

    value: T;

    return value;
}

#scope_export

// Frees the selected protobuf oneof case and resets its presence.
free_oneof :: (v: *$V) {
    #insert -> string {
        b: String_Builder;

        for V.case_names {
            info := V.case_infos[it_index];
            kind := "INT32";

            if info.type == .POINTER kind = "MESSAGE";
            else if info.type == .STRING kind = "STRING";
            else if info.type == .ARRAY kind = "BYTES";
            else continue;

            print(*b, "if cast(s64) v.which == % free_field(*v.%, .%);\n", it_index, it, kind);
        }

        return builder_to_string(*b);
    };

    v.* = .{};
}

// Reads a oneof occurrence without changing the selected case for a rejected closed enum.
read_oneof :: (r: *Reader, v: *$V, $selected: V.Which, $kind: Scalar, wire: u64, path: string) -> bool {
    if wire != scalar_wire(kind) return false;

    #insert -> string {
        name := V.case_names[cast(s64) selected];
        b: String_Builder;

        if kind == .MESSAGE || kind == .GROUP {
            print(*b, "if v.which != selected { release_oneof(r, v); v.which = selected; }\nreturn read_field(r, *v.%, kind, wire, path);\n", name);
        } else {
            print(*b, "item: type_of(v.%);\nif !read_scalar(r, *item, kind) return field_error(r, path);\n", name);
            if kind == .CLOSED_ENUM append(*b, "if r.enum_rejected return true;\n");
            print(*b, "release_oneof(r, v);\nv.which = selected;\nv.% = item;\nreturn true;\n", name);
        }

        return builder_to_string(*b);
    };
}

// Compares the selected case and its payload, with no field-name reflection at runtime.
equal_oneof :: (a: *$V, b: *V) -> bool {
    if a.which != b.which return false;

    #insert -> string {
        out: String_Builder;

        for V.case_names {
            info := V.case_infos[it_index];
            kind := "INT32";

            if info.type == .VOID continue;

            if info.type == .POINTER kind = "MESSAGE";
            else if info.type == .STRING kind = "STRING";
            else if info.type == .ARRAY kind = "BYTES";

            print(*out, "if cast(s64) a.which == % return equal_field(a.%, b.%, .%);\n", it_index, it, it, kind);
        }

        return builder_to_string(*out);
    };

    return true;
}
