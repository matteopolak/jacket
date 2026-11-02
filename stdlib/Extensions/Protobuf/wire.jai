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
}

// Decoder state shared by nested generated messages; end bounds only decrease for a child.
Reader :: struct {
    bytes: [] u8;
    pos: s64;
    end: s64 = -1;
    depth: s64;
    options: Decode_Options;
    error: Error;
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

// Writes a varint into the exact-sized output buffer.
write_varint :: (w: *Writer, input: u64) {
    value := input;
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

// Writes the low width bytes in little-endian order.
write_fixed :: (w: *Writer, value: u64, width: s64) {
    for i: 0..width - 1 w.bytes[w.pos + i] = cast(u8) ((value >> cast(u64) (8 * i)) & 255);
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
    tag := read_varint(r);

    if r.error.kind != .NONE return 0;

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

// Appends bytes to a sized output buffer.
write_bytes :: (w: *Writer, bytes: [] u8) {
    if bytes.count memcpy(w.bytes.data + w.pos, bytes.data, bytes.count);
    w.pos += bytes.count;
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

    r.end = r.pos + n;
    r.depth += 1;

    defer {
        r.end = old_end;
        r.depth -= 1;
    }

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
    } else #if kind == .INT32 || kind == .INT64 || kind == .ENUM || kind == .SFIXED32 || kind == .SFIXED64 return cast,trunc(u64) cast(s64) value;
    else return cast(u64) value;
}

// Wire type for a scalar kind.
scalar_wire :: (kind: Scalar) -> u64 {
    if kind == .MESSAGE || kind == .STRING || kind == .BYTES return 2;
    if kind == .FIXED64 || kind == .SFIXED64 || kind == .DOUBLE return 1;
    if kind == .FIXED32 || kind == .SFIXED32 || kind == .FLOAT return 5;
    return 0;
}

// Encoded scalar payload length, without its field tag.
scalar_size :: (value: $T, $kind: Scalar) -> s64 {
    #if kind == .STRING || kind == .BYTES return varint_size(cast(u64) value.count) + value.count;
    else {
        wire := scalar_wire(kind);
        if wire == 1 return 8;
        if wire == 5 return 4;
        return varint_size(scalar_bits(value, kind));
    }
}

// Writes a scalar payload without its tag.
write_scalar :: (w: *Writer, value: $T, $kind: Scalar) {
    #if kind == .STRING || kind == .BYTES {
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
    #if kind == .STRING || kind == .BYTES {
        Basic.free(value.data);
        value.* = .{};
    }
}

// Reads a scalar payload, replacing an owned string or byte buffer only after validation.
read_scalar :: (r: *Reader, value: *$T, $kind: Scalar) -> bool {
    #if kind == .STRING || kind == .BYTES {
        n := read_length(r);

        if r.error.kind != .NONE return false;

        bytes: [] u8 = .{ n, r.bytes.data + r.pos };

        #if kind == .STRING if !valid_utf8(bytes) return fail(r, .BAD_UTF8);
        free_scalar(value, kind);

        if !r.options.borrow {
            owned := NewArray(n, u8);
            if n memcpy(owned.data, bytes.data, n);
            bytes = owned;
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
        else value.* = cast,trunc(T) bits;
    }

    return true;
}
