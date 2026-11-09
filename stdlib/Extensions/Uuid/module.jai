// RFC 9562 UUIDs in network byte order. No owned memory is stored in a Uuid.
Uuid :: struct {
    bytes: [16] u8;
}

UUID_NIL :: Uuid.{};
UUID_MAX :: Uuid.{ .[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255] };

Uuid_Error_Kind :: enum u8 {
    NONE;
    LENGTH;
    SYNTAX;
    LIMIT;
    RANDOM;
    CLOCK;
    EXHAUSTED;
}

// Messages are static; offsets are zero-based bytes. No error needs freeing.
Uuid_Error :: struct {
    kind: Uuid_Error_Kind;
    message: string;
    offset: int;
}

Uuid_Variant :: enum u8 {
    NCS;
    RFC;
    MICROSOFT;
    FUTURE;
}

Uuid_Options :: struct {
    max_name_size := 1 << 20;
}

// Reads canonical hex, urn:uuid:hex, or {hex}; e.g. `id, err := parse_uuid(text)`.
// Fixed maximum input size is 45 bytes; parsing never allocates.
parse_uuid :: (text: string) -> Uuid, Uuid_Error {
    start := 0;
    size := text.count;

    if size == 45 && slice(text, 0, 9) == "urn:uuid:" start = 9;
    else if size == 38 && text[0] == #char "{" {
        if text[37] != #char "}" return .{}, .{ .SYNTAX, "expected closing brace", 37 };
        start = 1;
    } else if size != 36 return .{}, .{ .LENGTH, "expected a canonical UUID, URN, or braced UUID", clamp(size, 0, 45) };

    value: Uuid;
    nibble := 0;

    for i: 0..35 {
        c := text[start + i];

        if i == 8 || i == 13 || i == 18 || i == 23 {
            if c != #char "-" return .{}, .{ .SYNTAX, "expected hyphen", start + i };
        } else {
            n := hex_digit(c);
            if n < 0 return .{}, .{ .SYNTAX, "expected hexadecimal digit", start + i };
            value.bytes[nibble / 2] = (value.bytes[nibble / 2] << 4) | cast(u8) n;
            nibble += 1;
        }
    }

    return value, .{};
}

// Appends lowercase canonical text; e.g. `append_uuid(*builder, UUID_NIL)`.
append_uuid :: (builder: *String_Builder, value: Uuid) {
    bytes: [36] u8;
    format_into(bytes.data, value);
    append(builder, string.{ 36, bytes.data });
}

// Returns owned canonical text; e.g. `text := format_uuid(id); defer free(text)`.
format_uuid :: (value: Uuid, allocator := context.allocator) -> string {
    push_allocator(allocator);
    text := alloc_string(36);
    format_into(text.data, value);
    return text;
}

// Reads the version nibble even for reserved variants; e.g. `uuid_version(UUID_NIL) == 0`.
uuid_version :: (value: Uuid) -> u8 {
    return value.bytes[6] >> 4;
}

// Classifies the variant prefix; e.g. `uuid_variant(UUID_NIL) == .NCS`.
uuid_variant :: (value: Uuid) -> Uuid_Variant {
    b := value.bytes[8];
    if b < 128 return .NCS;
    if b < 192 return .RFC;
    if b < 224 return .MICROSOFT;

    return .FUTURE;
}

// Network-byte lexicographic order (-1/0/1); e.g. `uuid_compare(UUID_NIL, UUID_MAX) == -1`.
uuid_compare :: (a: Uuid, b: Uuid) -> int {
    for i: 0..15 {
        if a.bytes[i] < b.bytes[i] return -1;
        if a.bytes[i] > b.bytes[i] return 1;
    }

    return 0;
}

// Value equality; e.g. `Map.Map(Uuid, int, uuid_hash, uuid_equal)`.
uuid_equal :: (a: Uuid, b: Uuid) -> bool {
    for i: 0..15 if a.bytes[i] != b.bytes[i] return false;
    return true;
}

// Value equality operator; e.g. `id == UUID_NIL`.
operator == :: (a: Uuid, b: Uuid) -> bool {
    return uuid_equal(a, b);
}

// Hashes all 16 bytes; e.g. `Map.Map(Uuid, int, uuid_hash, uuid_equal)`.
uuid_hash :: (value: Uuid) -> u32 {
    return cast,trunc(u32) Hash.fnv1a_hash(value.bytes.data, 16);
}

// System CSPRNG v4; e.g. `id, err := uuid_v4()`; failure returns nil.
uuid_v4 :: () -> Uuid, Uuid_Error {
    value: Uuid;
    if !Crypto.random_bytes(value.bytes) return .{}, .{ .RANDOM, "system random generator failed", 0 };
    set_layout(*value, 4);
    return value, .{};
}

// Process-monotonic v7, RFC method 1; e.g. `id, err := uuid_v7()`.
// A 12-bit counter follows the timestamp; exhaustion returns an error without waiting.
uuid_v7 :: () -> Uuid, Uuid_Error {
    value: Uuid;
    if !Crypto.random_bytes(value.bytes) return .{}, .{ .RANDOM, "system random generator failed", 0 };

    ms, ok := to_milliseconds(current_time_consensus() - APOLLO_TIME_FOR_JAN_1_1970);
    if !ok return .{}, .{ .CLOCK, "clock is outside the millisecond range", 0 };

    return v7_advance(*v7_state, ms, value);
}

// MD5 name UUID for legacy interoperability; e.g. `id, err := uuid_v3(namespace, "name")`.
uuid_v3 :: #bake_arguments name_uuid(sha = false);

// SHA-1 name UUID; e.g. `id, err := uuid_v5(namespace, "name")`. Names are exact bytes.
uuid_v5 :: #bake_arguments name_uuid(sha = true);

// Json with=uuid_json encoder; e.g. `id: Uuid; @json(with=uuid_json)` with aliases in the struct.
uuid_json_encode :: (writer: *Json.Json_Writer, value: *Uuid) {
    bytes: [36] u8;
    format_into(bytes.data, value.*);
    Json.json_write_string(writer, .{ 36, bytes.data });
}

// Json with=uuid_json decoder; e.g. `uuid_json_decode :: U.uuid_json_decode` in the struct.
uuid_json_decode :: (reader: *Json.Json_Reader, out: *Uuid) -> bool {
    text, ok := Json.json_read_string(reader);
    if !ok return false;

    defer if !reader.borrow && text.data free(text);
    value, error := parse_uuid(text);
    if error.kind != .NONE return Json.json_fail(reader, .INVALID_VALUE, error.message);

    out.* = value;

    return true;
}

#scope_module
#import "Basic";
#import "String";
Hash :: #import "Hash";
Json :: #import "Extensions/Json";
Crypto :: #import,file "../../internal/crypto/random.jai";
Sha :: #import,file "../../internal/crypto/sha.jai";
Md5 :: #import "md5";

v7_state: u64;

hex_digit :: (c: u8) -> int {
    if c >= #char "0" && c <= #char "9" return c - #char "0";
    if c >= #char "a" && c <= #char "f" return c - #char "a" + 10;
    if c >= #char "A" && c <= #char "F" return c - #char "A" + 10;
    return -1;
}

format_into :: (out: *u8, value: Uuid) {
    at := 0;

    for i: 0..15 {
        if i == 4 || i == 6 || i == 8 || i == 10 {
            out[at] = #char "-";
            at += 1;
        }

        b := value.bytes[i];

        out[at] = "0123456789abcdef"[b >> 4];
        out[at + 1] = "0123456789abcdef"[b & 15];
        at += 2;
    }
}

set_layout :: (value: *Uuid, version: u8) {
    value.bytes[6] = (value.bytes[6] & 15) | (version << 4);
    value.bytes[8] = (value.bytes[8] & 63) | 128;
}

v7_advance :: (state: *u64, ms: s64, random: Uuid) -> Uuid, Uuid_Error {
    if ms < 0 || ms > 0xffffffffffff return .{}, .{ .CLOCK, "clock is outside the UUID timestamp range", 0 };

    stamp := cast(u64) ms;
    chosen: u64;

    // Timestamp and counter share one atomic word: concurrent callers cannot repeat a pair.
    while true {
        old := atomic_load(state);
        if old == 0 || stamp > (old >> 12) {
            seed := (((cast(u64) random.bytes[6]) << 8) | random.bytes[7]) & 1023;
            chosen = (stamp << 12) | (seed + 1);
        } else {
            if (old & 4095) == 4095 return .{}, .{ .EXHAUSTED, "UUID counter exhausted; clock must advance", 0 };
            chosen = old + 1;
        }

        swapped, _ := compare_and_swap(state, old, chosen);
        if swapped break;
    }

    value := random;

    stamp = chosen >> 12;
    for i: 0..5 value.bytes[5 - i] = cast,trunc(u8) (stamp >> (i * 8));

    value.bytes[6] = cast(u8) ((chosen >> 8) & 15);
    value.bytes[7] = cast,trunc(u8) chosen;
    set_layout(*value, 7);

    return value, .{};
}

name_uuid :: (namespace: Uuid, name: string, options := Uuid_Options.{}, sha: bool) -> Uuid, Uuid_Error {
    // The hard ceiling also keeps the legacy MD5 padding arithmetic safely in range.
    if name.count < 0 || name.count > min(options.max_name_size, 1 << 30) return .{}, .{ .LIMIT, "UUID name exceeds size limit", clamp(options.max_name_size, 0, 1 << 30) };

    input := alloc_string(16 + name.count);

    defer free(input);
    memcpy(input.data, namespace.bytes.data, 16);
    if name.count memcpy(input.data + 16, name.data, name.count);

    value: Uuid;

    if sha {
        digest := Sha.sha1(input);
        memcpy(value.bytes.data, digest.data, 16);
        set_layout(*value, 5);
    } else {
        digest := Md5.md5(input);
        defer free(digest);
        for i: 0..15 value.bytes[i] = cast(u8) ((hex_digit(digest[i * 2]) << 4) | hex_digit(digest[i * 2 + 1]));
        set_layout(*value, 3);
    }

    return value, .{};
}
