#scope_export

// Typed access to extension fields. An extension value is stored as wire bytes in the
// extendee's `_unknown`, so decoding, encoding, merging and equality need no extra code.
// The helpers below scan and rewrite those bytes.

// A singular extension of message `Extendee` holding a `T`. The Schema module generates one
// constant per `extend` field; `kind` selects the wire form. Without a value `get` returns
// `default`.
Extension :: struct(Extendee: Type, T: Type, kind: Scalar) {
    // The field number.
    number: s64;
    // The full protobuf name, used in error paths.
    name: string;
    // The value of an absent extension (the schema default).
    default: T;
}

// A repeated extension of message `Extendee` with elements of type `T`.
Repeated_Extension :: struct(Extendee: Type, T: Type, kind: Scalar) {
    // The field number.
    number: s64;
    // The full protobuf name, used in error paths.
    name: string;
    // Whether `add` and `set` write numeric elements packed.
    packed: bool;
}

// Reads a singular extension. The last occurrence of a scalar wins, and message occurrences
// merge. Returns the default and false when absent. Strings, bytes and the strings inside a
// message point into the extendee's unknown fields, so they stay valid until it is changed,
// freed or decoded into. A message value owns blocks: release it with `free`.
get :: (message: *$E, extension: Extension(E, $T, $kind)) -> T, bool {
    value := extension.default;
    found := false;

    #if kind == .MESSAGE || kind == .GROUP value = .{};

    r := Reader.{ bytes = message._unknown, options = .{ borrow = true, skip_utf8 = true } };

    #if kind == .MESSAGE || kind == .GROUP r.storage = *value.proto_storage;

    while true {
        tag, ok := next_extension_field(*r, extension.number);
        if !ok break;

        if tag & 7 != scalar_wire(kind) {
            if !skip(*r, tag) break;
            continue;
        }

        #if kind == .MESSAGE || kind == .GROUP {
            if !read_extension_value(*r, *value, kind, extension.number) {
                free_message(*value);
                value = .{};
                found = false;
                break;
            }

            found = true;
        } else {
            item: T;
            if !read_extension_value(*r, *item, kind, extension.number) break;

            #if kind == .CLOSED_ENUM if r.enum_rejected continue;

            value = item;
            found = true;
        }
    }

    Basic.free(r.error.path);

    return value, found;
}

// Reads every element of a repeated extension in wire order; packed and expanded forms both
// count. The result is empty when absent. Release it with `free`. Elements follow the
// lifetime rules of `get`.
get :: (message: *$E, extension: Repeated_Extension(E, $T, $kind)) -> [..] T {
    PACKABLE :: #run packable_kind(kind);
    values: [..] T;
    r := Reader.{ bytes = message._unknown, options = .{ borrow = true, skip_utf8 = true } };

    while true {
        tag, ok := next_extension_field(*r, extension.number);
        if !ok break;

        wire := tag & 7;

        #if PACKABLE {
            if wire == 2 {
                n := read_length(*r);

                if r.error.kind != .NONE break;

                old_end := r.end;

                r.end = r.pos + n;

                while remaining(*r) > 0 {
                    item: T;
                    if !read_scalar(*r, *item, kind) break;
                    #if kind == .CLOSED_ENUM if r.enum_rejected continue;
                    array_add(*values, item);
                }

                r.end = old_end;
                if r.error.kind != .NONE break;

                continue;
            }
        }

        if wire != scalar_wire(kind) {
            if !skip(*r, tag) break;
            continue;
        }

        item: T;

        #if kind == .MESSAGE || kind == .GROUP r.storage = *item.proto_storage;

        if !read_extension_value(*r, *item, kind, extension.number) {
            #if kind == .MESSAGE || kind == .GROUP free_message(*item);
            break;
        }

        #if kind == .CLOSED_ENUM if r.enum_rejected continue;

        array_add(*values, item);
    }

    Basic.free(r.error.path);

    return values;
}

// Releases the result of `get` on a repeated extension, including message elements.
free :: (values: *[..] $T) {
    #if type_info(T).type == .STRUCT for *values.* T.proto_free(it);
    array_free(values.*);
    values.* = .{};
}

// Tests whether a singular extension is present.
has :: (message: *$E, extension: Extension(E, $T, $kind)) -> bool {
    return has_extension(message._unknown, extension.number, kind, false);
}

// Tests whether a repeated extension has any element.
has :: (message: *$E, extension: Repeated_Extension(E, $T, $kind)) -> bool {
    return has_extension(message._unknown, extension.number, kind, true);
}

// Sets a singular extension, replacing every earlier occurrence. A message value is encoded
// (copied) and stays the caller's; it fails like `encode` when a required field is missing,
// leaving the extendee unchanged.
set :: (message: *$E, extension: Extension(E, $T, $kind), value: T) -> Error {
    field: [..] u8;

    defer array_free(field);

    error := encode_extension(*field, extension.number, value, kind);
    if error.kind != .NONE return error;

    replace_extension(message, extension.number, kind, false, field);

    return .{};
}

// Replaces all elements of a repeated extension.
set :: (message: *$E, extension: Repeated_Extension(E, $T, $kind), values: [] T) -> Error {
    field: [..] u8;

    defer array_free(field);

    error := encode_extension_values(*field, extension, values);
    if error.kind != .NONE return error;

    replace_extension(message, extension.number, kind, true, field);

    return .{};
}

// Appends one element to a repeated extension.
add :: (message: *$E, extension: Repeated_Extension(E, $T, $kind), value: T) -> Error {
    field: [..] u8;

    defer array_free(field);

    item := value;
    error := encode_extension_values(*field, extension, .{ 1, *item });

    if error.kind != .NONE return error;

    array_add(*message._unknown, ..field);

    return .{};
}

// Removes a singular extension.
clear :: (message: *$E, extension: Extension(E, $T, $kind)) {
    remove_extension(*message._unknown, extension.number, kind, false);
}

// Removes every element of a repeated extension.
clear :: (message: *$E, extension: Repeated_Extension(E, $T, $kind)) {
    remove_extension(*message._unknown, extension.number, kind, true);
}

// Validates an extension value met while decoding, as a normal field of that type would be:
// a string must be UTF-8, and a message must decode. Called from the extendee's generated
// decoder only for extensions that need it; `skip_utf8` turns the check off.
check_extension :: (r: *Reader, tag: u64, $kind: Scalar, $T: Type, name: string) -> bool {
    if r.options.skip_utf8 || tag & 7 != scalar_wire(kind) return true;

    sub := Reader.{ bytes = r.bytes, pos = r.tag_start, options = r.options, depth = r.depth };
    sub.options.borrow = true;
    read_tag(*sub);

    #if kind == .STRING {
        n := read_length(*sub);

        if sub.error.kind != .NONE return true;

        if !valid_utf8(.{ n, sub.bytes.data + sub.pos }) {
            fail(*sub, .BAD_UTF8);
            r.error = sub.error;
            return field_error(r, tprint("[%]", name));
        }
    } else {
        item: T;

        sub.storage = *item.proto_storage;

        ok: bool;

        #if kind == .GROUP {
            sub.enum_tag = tag & ~7;
            ok = read_group(*sub, *item);
        } else ok = read_message(*sub, *item);

        free_message(*item);

        if !ok {
            r.error = sub.error;
            return field_error(r, tprint("[%]", name));
        }
    }

    return true;
}

#scope_module

Mem :: #import "Memory";

// Numeric kinds may be written packed.
packable_kind :: (kind: Scalar) -> bool {
    return kind != .MESSAGE && kind != .GROUP && kind != .STRING && kind != .RAW_STRING && kind != .BYTES;
}

// Whether a field of this wire type is an occurrence of the extension.
extension_accepts :: (kind: Scalar, repeated: bool, wire: u64) -> bool {
    if wire == scalar_wire(kind) return true;
    return repeated && wire == 2 && kind != .MESSAGE && kind != .GROUP && kind != .STRING && kind != .RAW_STRING && kind != .BYTES;
}

// Moves to the next field with this number, leaving the reader after its tag.
next_extension_field :: (r: *Reader, number: s64) -> u64, bool {
    while remaining(r) > 0 {
        r.tag_start = r.pos;
        tag := read_tag(r);

        if r.error.kind != .NONE return 0, false;
        if cast(s64) (tag >> 3) == number return tag, true;
        if !skip(r, tag) return 0, false;
    }

    return 0, false;
}

// Reads one occurrence at the reader, which sits after the tag. Strings and bytes become
// views of the input; a message merges into the value.
read_extension_value :: (r: *Reader, value: *$T, $kind: Scalar, number: s64) -> bool {
    #if kind == .STRING || kind == .RAW_STRING || kind == .BYTES {
        n := read_length(r);

        if r.error.kind != .NONE return false;

        value.count = n;
        value.data = r.bytes.data + r.pos;
        r.pos += n;

        return true;
    } else #if kind == .MESSAGE {
        return read_message(r, value);
    } else #if kind == .GROUP {
        r.enum_tag = cast(u64) number * 8;
        return read_group(r, value);
    } else return read_scalar(r, value, kind);
}

// Whether any field of the number has a wire type the extension accepts.
has_extension :: (bytes: [] u8, number: s64, $kind: Scalar, repeated: bool) -> bool {
    r := Reader.{ bytes = bytes };

    while true {
        tag, ok := next_extension_field(*r, number);
        if !ok return false;
        if extension_accepts(kind, repeated, tag & 7) return true;
        if !skip(*r, tag) return false;
    }

    return false;
}

// Deletes the matching occurrences in place; a malformed tail is kept as it is.
remove_extension :: (unknown: *[..] u8, number: s64, $kind: Scalar, repeated: bool) {
    r := Reader.{ bytes = unknown.* };
    out: s64;
    good: s64;

    while remaining(*r) > 0 {
        start := r.pos;
        tag := read_tag(*r);

        if r.error.kind != .NONE break;
        if !skip(*r, tag) break;

        good = r.pos;
        if cast(s64) (tag >> 3) == number && extension_accepts(kind, repeated, tag & 7) continue;

        if out != start Mem.copy(unknown.data + out, unknown.data + start, good - start);

        out += good - start;
    }

    tail := unknown.count - good;
    if tail && out != good Mem.copy(unknown.data + out, unknown.data + good, tail);

    unknown.count = out + tail;
}

// Swaps in an encoded field after dropping the earlier occurrences. The new bytes were
// encoded first because a string or message value may point into the old ones.
replace_extension :: (message: *$E, number: s64, $kind: Scalar, repeated: bool, field: [] u8) {
    remove_extension(*message._unknown, number, kind, repeated);
    array_add(*message._unknown, ..field);
}

// Appends tag and payload of one value to `out`.
encode_extension :: (out: *[..] u8, number: s64, value: $T, $kind: Scalar) -> Error {
    tag := cast(u64) number * 8 + scalar_wire(kind);
    start := out.count;

    #if kind == .MESSAGE || kind == .GROUP {
        bytes, error := encode(value);
        if error.kind != .NONE return error;

        defer Basic.free(bytes.data);

        n := varint_size(tag) + bytes.count;

        #if kind == .GROUP n += varint_size(tag + 1);
        else n += varint_size(cast(u64) bytes.count);

        array_reserve(out, start + n);

        w := Writer.{ bytes = .{ n, out.data + start } };

        write_varint(*w, tag);
        #if kind != .GROUP write_varint(*w, cast(u64) bytes.count);
        write_bytes(*w, bytes);
        #if kind == .GROUP write_varint(*w, tag + 1);
    } else {
        n := varint_size(tag) + scalar_size(value, kind);

        array_reserve(out, start + n);

        w := Writer.{ bytes = .{ n, out.data + start } };

        write_varint(*w, tag);
        write_scalar(*w, value, kind);
    }

    out.count = start + n;

    return .{};
}

// Appends the elements of a repeated extension: one packed field for numbers when the
// extension is packed, otherwise one field per element.
encode_extension_values :: (out: *[..] u8, extension: Repeated_Extension($E, $T, $kind), values: [] T) -> Error {
    PACKABLE :: #run packable_kind(kind);

    #if PACKABLE {
        if extension.packed {
            if !values.count return .{};

            tag := cast(u64) extension.number * 8 + 2;
            payload: s64;

            for values payload += scalar_size(it, kind);

            start := out.count;
            n := varint_size(tag) + varint_size(cast(u64) payload) + payload;

            array_reserve(out, start + n);

            w := Writer.{ bytes = .{ n, out.data + start } };

            write_varint(*w, tag);
            write_varint(*w, cast(u64) payload);
            for values write_scalar(*w, it, kind);

            out.count = start + n;

            return .{};
        }
    }

    for values {
        error := encode_extension(out, extension.number, it, kind);
        if error.kind != .NONE return error;
    }

    return .{};
}
