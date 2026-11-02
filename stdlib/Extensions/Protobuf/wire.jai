// Wire kinds specialized by generated field operations.
Scalar :: enum {
    INT32;
    INT64;
    UINT32;
    UINT64;
    SINT32;
    SINT64;
    FIXED32;
    FIXED64;
    SFIXED32;
    SFIXED64;
    FLOAT;
    DOUBLE;
    BOOL;
    STRING;
    BYTES;
    ENUM;
    MESSAGE;
    GROUP;
    CLOSED_ENUM;
    RAW_STRING;
}

// Decoder state shared by nested generated messages; end bounds only decrease for a child.
Reader :: struct {
    bytes: [] u8;
    pos: s64;
    end: s64 = -1;
    depth: s64;
    options: Decode_Options;
    error: Error;
    group: u64;
    group_ended: bool;
    unknown: *[..] u8;
    tag_start: s64;
    enum_tag: u64;
    enum_rejected: bool;
    suppress_enum_unknown: bool;
    storage: **Storage_Block;
    payload: [] u8;
    chunk_size: s64;
    borrow_recorded: bool;
}

// Exact output buffer and preorder nested-message sizes used by generated encoders.
Writer :: struct {
    bytes: [] u8;
    pos: s64;
    cache: [] u32;
    next: s64;
}

// Records the first failure; generated code adds path segments while unwinding.
fail :: (r: *Reader, kind: Error_Kind) -> bool {
    if r.error.kind == .NONE r.error = .{ kind = kind, offset = r.pos };
    return false;
}

// Prepends a field path to a nested error. Called only on failure.
field_error :: (r: *Reader, name: string, index: s64 = -1) -> bool {
    prefix := name;
    if index >= 0 prefix = tprint("%[%]", name, index);

    old := r.error.path;
    if old.count r.error.path = sprint("%.%", prefix, old);
    else r.error.path = copy_string(prefix);

    Basic.free(old);

    return false;
}

// Remaining bytes within the active message or packed field.
remaining :: (r: *Reader) -> s64 {
    if r.end < 0 return r.bytes.count - r.pos;
    return r.end - r.pos;
}

// Reads a bounded unsigned varint, rejecting overflow in the tenth byte.
read_varint :: (r: *Reader) -> u64 {
    result: u64;

    // With ten bytes in view a varint cannot run off the end, so one check covers every byte.
    if remaining(r) >= 10 {
        p := r.bytes.data + r.pos;
        byte := p.*;

        if byte < 128 {
            r.pos += 1;
            return cast(u64) byte;
        }

        result = cast(u64) (byte & 127);

        for 1..8 {
            byte = (p + it).*;
            result |= (cast(u64) (byte & 127)) << cast(u64) (7 * it);

            if byte < 128 {
                r.pos += it + 1;
                return result;
            }
        }

        byte = (p + 9).*;
        r.pos += 10;

        if byte > 1 {
            fail(r, .VARINT_TOO_LONG);
            return 0;
        }

        return result | ((cast(u64) byte) << 63);
    }

    {
        if remaining(r) <= 0 {
            fail(r, .TRUNCATED);
            return 0;
        }

        byte := r.bytes[r.pos];

        r.pos += 1;
        result |= (cast(u64) (byte & 127)) << 0;
        if byte < 128 return result;
    }

    {
        if remaining(r) <= 0 {
            fail(r, .TRUNCATED);
            return 0;
        }

        byte := r.bytes[r.pos];

        r.pos += 1;
        result |= (cast(u64) (byte & 127)) << 7;
        if byte < 128 return result;
    }

    {
        if remaining(r) <= 0 {
            fail(r, .TRUNCATED);
            return 0;
        }

        byte := r.bytes[r.pos];

        r.pos += 1;
        result |= (cast(u64) (byte & 127)) << 14;
        if byte < 128 return result;
    }

    {
        if remaining(r) <= 0 {
            fail(r, .TRUNCATED);
            return 0;
        }

        byte := r.bytes[r.pos];

        r.pos += 1;
        result |= (cast(u64) (byte & 127)) << 21;
        if byte < 128 return result;
    }

    {
        if remaining(r) <= 0 {
            fail(r, .TRUNCATED);
            return 0;
        }

        byte := r.bytes[r.pos];

        r.pos += 1;
        result |= (cast(u64) (byte & 127)) << 28;
        if byte < 128 return result;
    }

    {
        if remaining(r) <= 0 {
            fail(r, .TRUNCATED);
            return 0;
        }

        byte := r.bytes[r.pos];

        r.pos += 1;
        result |= (cast(u64) (byte & 127)) << 35;
        if byte < 128 return result;
    }

    {
        if remaining(r) <= 0 {
            fail(r, .TRUNCATED);
            return 0;
        }

        byte := r.bytes[r.pos];

        r.pos += 1;
        result |= (cast(u64) (byte & 127)) << 42;
        if byte < 128 return result;
    }

    {
        if remaining(r) <= 0 {
            fail(r, .TRUNCATED);
            return 0;
        }

        byte := r.bytes[r.pos];

        r.pos += 1;
        result |= (cast(u64) (byte & 127)) << 49;
        if byte < 128 return result;
    }

    {
        if remaining(r) <= 0 {
            fail(r, .TRUNCATED);
            return 0;
        }

        byte := r.bytes[r.pos];

        r.pos += 1;
        result |= (cast(u64) (byte & 127)) << 56;
        if byte < 128 return result;
    }

    {
        if remaining(r) <= 0 {
            fail(r, .TRUNCATED);
            return 0;
        }

        byte := r.bytes[r.pos];

        r.pos += 1;

        if byte > 1 {
            fail(r, .VARINT_TOO_LONG);
            return 0;
        }

        result |= (cast(u64) (byte & 127)) << 63;
        if byte < 128 return result;
    }

    fail(r, .VARINT_TOO_LONG);

    return 0;
}

// Number of bytes in an unsigned varint.
varint_size :: (value: u64) -> s64 {
    if value < 128 return 1;
    if value < 16384 return 2;
    if value < 2097152 return 3;
    if value < 268435456 return 4;
    if value < 34359738368 return 5;
    if value < 4398046511104 return 6;
    if value < 562949953421312 return 7;
    if value < 72057594037927936 return 8;
    if value < 9223372036854775808 return 9;

    return 10;
}

// Writes a varint into the exact-sized output buffer. One-byte values (tags, short
// lengths, small numbers) stay inline at the call site.
write_varint :: inline (w: *Writer, value: u64) {
    if value < 128 {
        w.bytes.data[w.pos] = cast(u8) value;
        w.pos += 1;
    } else write_long_varint(w, value);
}

// The multi-byte path of write_varint.
write_long_varint :: (w: *Writer, input: u64) {
    value := input;

    // Room for the longest varint means the loop below needs no per-byte check.
    if w.bytes.count - w.pos >= 10 {
        p := w.bytes.data + w.pos;

        while value >= 128 {
            p.* = cast(u8) ((value & 127) | 128);
            p += 1;
            value >>= 7;
        }

        p.* = cast(u8) value;
        w.pos = (p - w.bytes.data) + 1;

        return;
    }

    while value >= 128 {
        w.bytes[w.pos] = cast(u8) ((value & 127) | 128);
        w.pos += 1;
        value >>= 7;
    }

    w.bytes[w.pos] = cast(u8) value;
    w.pos += 1;
}

// Zigzag maps the signed 64-bit extremes to unsigned varints without signed overflow.
zigzag :: (value: s64) -> u64 {
    return ((cast,trunc(u64) value) << 1) ^ (cast,trunc(u64) (value >> 63));
}

// Inverse zigzag. For sint32 callers truncate to u32 before calling.
unzigzag :: (value: u64) -> s64 {
    return (cast(s64) (value >> 1)) ^ (-cast(s64) (value & 1));
}

// Reads a little-endian fixed-width integer without unaligned loads.
read_fixed :: (r: *Reader, width: s64) -> u64 {
    if remaining(r) < width {
        fail(r, .TRUNCATED);
        return 0;
    }

    result: u64;
    for i: 0..width - 1 result |= (cast(u64) r.bytes[r.pos + i]) << cast(u64) (8 * i);

    r.pos += width;

    return result;
}

// Writes the low width bytes in little-endian order. Every supported target (x64, arm64,
// wasm) is little endian, so this is one store.
write_fixed :: inline (w: *Writer, value: u64, $width: s64) {
    bits := value;
    memcpy(w.bytes.data + w.pos, *bits, width);
    w.pos += width;
}

// Reads and checks a length before any slice or allocation. Size limits precede truncation.
read_length :: (r: *Reader) -> s64 {
    n := read_varint(r);

    if r.error.kind != .NONE return 0;

    if n > cast(u64) max(0, r.options.max_size) {
        fail(r, .TOO_LARGE);
        return 0;
    }

    if n > cast(u64) remaining(r) {
        fail(r, .TRUNCATED);
        return 0;
    }

    return cast(s64) n;
}

// Validates a protobuf tag before narrowing it.
read_tag :: (r: *Reader) -> u64 {
    start := r.pos;
    tag := read_varint(r);

    if r.error.kind != .NONE return 0;

    // A tag is a 32-bit varint: at most 5 bytes, even when the extra bytes are zero padding.
    if r.pos - start > 5 {
        fail(r, .BAD_TAG);
        return 0;
    }

    if tag >> 3 == 0 {
        fail(r, .FIELD_ZERO);
        return 0;
    }

    if tag >> 3 > 536870911 {
        fail(r, .BAD_TAG);
        return 0;
    }

    if (tag & 7) >= 6 {
        fail(r, .BAD_WIRE_TYPE);
        return 0;
    }

    return tag;
}

// Skips an unknown wire field, including a matching group as one opaque unit.
skip :: (r: *Reader, tag: u64) -> bool {
    wire := tag & 7;
    if wire == 0 read_varint(r);
    else if wire == 1 read_fixed(r, 8);
    else if wire == 2 {
        n := read_length(r);
        if r.error.kind == .NONE r.pos += n;
    } else if wire == 5 read_fixed(r, 4);
    else if wire == 4 return fail(r, .BAD_GROUP);
    else if wire == 3 {
        if r.depth >= r.options.max_depth return fail(r, .TOO_DEEP);

        r.depth += 1;
        defer r.depth -= 1;

        while remaining(r) > 0 {
            inner := read_tag(r);

            if r.error.kind != .NONE return false;

            if (inner & 7) == 4 {
                if inner >> 3 != tag >> 3 return fail(r, .BAD_GROUP);
                return true;
            }

            if !skip(r, inner) return false;
        }

        return fail(r, .TRUNCATED);
    }

    return r.error.kind == .NONE;
}

// Allocates byte storage whose caller must completely initialize before exposing it.
allocate_bytes :: (count: s64) -> [] u8 {
    if !count return .{};
    data := cast(*u8) alloc(count);
    assert(data != null);
    return .{ count, data };
}

// Appends bytes to a sized output buffer. Up to 16 bytes are copied with two overlapping
// fixed-size moves, which compile to plain loads and stores instead of a memcpy call.
write_bytes :: inline (w: *Writer, bytes: [] u8) {
    n := bytes.count;
    to := w.bytes.data + w.pos;
    from := bytes.data;

    if n > 16 memcpy(to, from, n);
    else if n >= 8 {
        memcpy(to, from, 8);
        memcpy(to + n - 8, from + n - 8, 8);
    } else if n >= 4 {
        memcpy(to, from, 4);
        memcpy(to + n - 4, from + n - 4, 4);
    } else if n {
        to[0] = from[0];
        to[n / 2] = from[n / 2];
        to[n - 1] = from[n - 1];
    }

    w.pos += n;
}

// Writes a message using its preorder size entry (no back-patching).
write_message :: (w: *Writer, value: *$T) {
    w.next += 1;
    T.proto_encode(value, w);
}

// Decodes a length-delimited child into an existing message, restoring parent bounds.
read_message :: (r: *Reader, value: *$T) -> bool {
    n := read_length(r);

    if r.error.kind != .NONE return false;
    if r.depth >= r.options.max_depth return fail(r, .TOO_DEEP);

    old_end := r.end;
    old_group := r.group;
    old_ended := r.group_ended;

    r.group = 0;
    r.group_ended = false;

    r.end = r.pos + n;
    r.depth += 1;

    defer {
        r.end = old_end;
        r.group = old_group;
        r.group_ended = old_ended;
        r.depth -= 1;
    }

    if r.storage absorb_storage(r, *value.proto_storage);

    return T.proto_merge(value, r);
}

// Compares byte sequences by content.
bytes_equal :: (a: [] u8, b: [] u8) -> bool {
    if a.count != b.count return false;
    for a if it != b[it_index] return false;
    return true;
}

// Strict UTF-8 validation: rejects overlong encodings, surrogate code points and > U+10FFFF.
valid_utf8 :: (bytes: [] u8) -> bool {
    i: s64;
    // Skip the ASCII prefix in complete unaligned words, then use the strict scalar
    // validator for the tail and non-ASCII text. Short strings pay one prefix check.
    while bytes.count - i >= 8 {
        word: u64;
        memcpy(*word, bytes.data + i, 8);
        if (word & 0x8080808080808080) != 0 break;
        i += 8;
    }

    if bytes.count - i >= 4 {
        word: u32;
        memcpy(*word, bytes.data + i, 4);
        if (word & 0x80808080) == 0 i += 4;
    }

    while i < bytes.count {
        a := bytes[i];

        i += 1;
        if a < 128 continue;

        count: s64;
        low: u8 = 128;
        high: u8 = 191;

        if a >= 194 && a <= 223 count = 1;
        else if a >= 224 && a <= 239 {
            count = 2;
            if a == 224 low = 160;
            if a == 237 high = 159;
        } else if a >= 240 && a <= 244 {
            count = 3;
            if a == 240 low = 144;
            if a == 244 high = 143;
        } else return false;

        if bytes.count - i < count return false;
        if bytes[i] < low || bytes[i] > high return false;
        for j: 1..count - 1 if bytes[i + j] < 128 || bytes[i + j] > 191 return false;

        i += count;
    }

    return true;
}

// Maps a scalar value to its unsigned wire bits; kind is specialized at compile time.
scalar_bits :: (value: $T, $kind: Scalar) -> u64 {
    #if kind == .SINT32 || kind == .SINT64 return zigzag(cast(s64) value);
    else #if kind == .FLOAT return cast(u64) (cast(*u32) *value).*;
    else #if kind == .DOUBLE return (cast(*u64) *value).*;
    else #if kind == .BOOL {
        if value return 1;
        else return 0;
    } else #if kind == .INT32 || kind == .INT64 || kind == .ENUM || kind == .CLOSED_ENUM || kind == .SFIXED32 || kind == .SFIXED64 return cast,trunc(u64) cast(s64) value;
    else return cast(u64) value;
}

// Wire type for a scalar kind.
scalar_wire :: (kind: Scalar) -> u64 {
    if kind == .GROUP return 3;
    if kind == .MESSAGE || kind == .STRING || kind == .RAW_STRING || kind == .BYTES return 2;
    if kind == .FIXED64 || kind == .SFIXED64 || kind == .DOUBLE return 1;
    if kind == .FIXED32 || kind == .SFIXED32 || kind == .FLOAT return 5;

    return 0;
}

// Encoded scalar payload length, without its field tag.
scalar_size :: (value: $T, $kind: Scalar) -> s64 {
    #if kind == .STRING || kind == .RAW_STRING || kind == .BYTES return varint_size(cast(u64) value.count) + value.count;
    else {
        wire := scalar_wire(kind);
        if wire == 1 return 8;
        if wire == 5 return 4;
        return varint_size(scalar_bits(value, kind));
    }
}

// Writes a scalar payload without its tag.
write_scalar :: (w: *Writer, value: $T, $kind: Scalar) {
    #if kind == .STRING || kind == .RAW_STRING || kind == .BYTES {
        write_varint(w, cast(u64) value.count);
        write_bytes(w, .{ value.count, value.data });
    } else {
        bits := scalar_bits(value, kind);
        wire := scalar_wire(kind);

        if wire == 1 write_fixed(w, bits, 8);
        else if wire == 5 write_fixed(w, bits, 4);
        else write_varint(w, bits);
    }
}

// Releases an owned scalar payload; numeric values need no allocation.
free_scalar :: (value: *$T, $kind: Scalar) {
    #if kind == .STRING || kind == .RAW_STRING || kind == .BYTES {
        free_owned_pointer(value.data);
        value.* = .{};
    }
}

// Reads a scalar payload, replacing an owned string or byte buffer only after validation.
read_scalar :: (r: *Reader, value: *$T, $kind: Scalar) -> bool {
    r.enum_rejected = false;
    start := r.pos;

    #if kind == .STRING || kind == .RAW_STRING || kind == .BYTES {
        n := read_length(r);

        if r.error.kind != .NONE return false;

        bytes: [] u8 = .{ n, r.bytes.data + r.pos };

        #if kind == .STRING if !r.options.skip_utf8 && !valid_utf8(bytes) return fail(r, .BAD_UTF8);
        release_scalar(r, value, kind);

        if !r.options.borrow {
            // The following copy initializes every byte; avoid clearing it first.
            owned := payload_bytes(r, n);
            if n memcpy(owned.data, bytes.data, n);
            bytes = owned;
        } else if !r.borrow_recorded {
            add_borrowed(r);
            r.borrow_recorded = true;
        }

        value.count = n;
        value.data = bytes.data;
        r.pos += n;
    } else {
        wire := scalar_wire(kind);
        bits: u64;

        if wire == 1 bits = read_fixed(r, 8);
        else if wire == 5 bits = read_fixed(r, 4);
        else bits = read_varint(r);
        if r.error.kind != .NONE return false;

        #if kind == .SINT32 value.* = cast(T) unzigzag(cast,trunc(u32) bits);
        else #if kind == .SINT64 value.* = cast(T) unzigzag(bits);
        else #if kind == .FLOAT {
            narrow := cast(u32) bits;
            value.* = (cast(*T) *narrow).*;
        } else #if kind == .DOUBLE value.* = (cast(*T) *bits).*;
        else #if kind == .BOOL value.* = bits != 0;
        else {
            candidate := cast,trunc(T) bits;
            #if kind == .CLOSED_ENUM if !enum_known(candidate) {
                r.enum_rejected = true;
                if !r.suppress_enum_unknown && !r.options.discard_unknown && r.unknown {
                    tag := r.enum_tag;
                    while tag >= 128 {
                        append_unknown_byte(r, cast(u8) ((tag & 127) | 128));
                        tag >>= 7;
                    }

                    append_unknown_byte(r, cast(u8) tag);
                    raw: [] u8 = .{ r.pos - start, r.bytes.data + start };

                    append_unknown(r, raw);
                }

                return true;
            }

            value.* = candidate;
        }
    }

    return true;
}

// Tests declared enum numbers; specialization makes the membership table constant.
enum_known :: (value: $T) -> bool {
    #insert -> string {
        info := cast(*Type_Info_Enum) type_info(T);
        b: String_Builder;
        for info.values print(*b, "if cast(s32) value == % return true;\n", it);
        return builder_to_string(*b);
    };

    return false;
}

// Reads a typed group with a matching end tag and the same recursion budget as messages.
read_group :: (r: *Reader, value: *$T) -> bool {
    if r.depth >= r.options.max_depth return fail(r, .TOO_DEEP);

    old := r.group;
    ended := r.group_ended;

    r.group = r.enum_tag >> 3;
    r.group_ended = false;
    r.depth += 1;

    defer {
        r.group = old;
        r.group_ended = ended;
        r.depth -= 1;
    }

    if r.storage absorb_storage(r, *value.proto_storage);
    if !T.proto_merge(value, r) return false;
    if !r.group_ended return fail(r, .TRUNCATED);

    return true;
}

// Recursively checks required presence after merging all occurrences.
check_message :: (value: *$T, r: *Reader) -> bool {
    if !value return true;
    if r.depth >= r.options.max_depth return fail(r, .TOO_DEEP);

    r.depth += 1;
    defer r.depth -= 1;

    #if T.proto_has_required return T.proto_check(value, r);
    else return true;
}

// Constructs special floating schema defaults during generation, preserving IEEE values.
float_default :: (negative: bool, nan: bool) -> float64 {
    bits: u64 = 0x7ff0000000000000;
    if negative bits |= 0x8000000000000000;
    if nan bits |= 0x8000000000000;
    return (cast(*float64) *bits).*;
}

// Completes a typed group only when its end tag matches the active field.
end_group :: (r: *Reader, tag: u64) -> bool {
    if !r.group || tag >> 3 != r.group return fail(r, .BAD_GROUP);
    r.group_ended = true;
    return true;
}

// Skips and retains an unmatched complete field from the current message.
skip_unknown :: (r: *Reader, tag: u64) -> bool {
    start := r.tag_start;

    if !skip(r, tag) return false;

    if !r.options.discard_unknown && r.unknown {
        raw: [] u8 = .{ r.pos - start, r.bytes.data + start };
        append_unknown(r, raw);
    }

    return true;
}
