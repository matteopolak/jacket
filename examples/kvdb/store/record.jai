// The record format. Every change to the store is one record appended to the log:
//
//     0        4      5          9          13
//     +--------+------+----------+----------+-----+-------+
//     | check  | kind | key size | val size | key | value |
//     +--------+------+----------+----------+-----+-------+
//
// Integers are little-endian. `check` covers everything after it, so a record that was cut short
// by a crash, or that was damaged later, no longer matches its own checksum.

Record_Kind :: enum u8 {
    PUT :: 1;
    DELETE :: 2;
}

HEADER_SIZE :: 13;
MAX_KEY_SIZE :: 1024;
MAX_VALUE_SIZE :: 1 << 20;

// What `decode_record` found at an offset of the log.
Decode_Status :: enum {
    // A whole record with a matching checksum.
    OK;
    // The log ends inside the record: a write that never finished.
    TORN;
    // The record is complete but its bytes do not match the checksum.
    BAD_CHECKSUM;
    // The header cannot be a record of ours: an unknown kind or impossible sizes. Nothing says how
    // long it is, so the rest of the log cannot be read.
    BAD_HEADER;
}

Record :: struct {
    kind: Record_Kind;
    // These point into the buffer that was decoded; copy them to keep them.
    key: string;
    value: string;
    // Bytes the record takes in the log.
    size: s64;
}

// Keys are 1 to `MAX_KEY_SIZE` bytes with no space or control character in them, so that a key can
// be written on a command line or on a line of the protocol without quoting.
valid_key :: (key: string) -> bool {
    if key.count == 0 || key.count > MAX_KEY_SIZE return false;

    for key {
        if it <= #char " " return false;
    }

    return true;
}

record_size :: (key: string, value: string) -> s64 {
    return HEADER_SIZE + key.count + value.count;
}

// The checksum of `bytes`: the first four bytes of their MD5 digest. MD5 is no defence against an
// attacker, but a damaged record matches a 32-bit check by chance only one time in four billion.
checksum :: (bytes: string) -> u32 {
    hex := md5(bytes,, temp);
    value: u32;

    for 0..7 {
        digit := hex[it];
        nibble := ifx digit >= #char "a" then digit - #char "a" + 10 else digit - #char "0";

        value = (value << 4) | nibble;
    }

    return value;
}

// Writes a record into newly allocated memory. Free `.data` when done.
encode_record :: (kind: Record_Kind, key: string, value: string) -> string {
    bytes := alloc_string(record_size(key, value));

    bytes[4] = cast(u8) kind;
    put_u32(bytes, 5, cast(u32) key.count);
    put_u32(bytes, 9, cast(u32) value.count);
    memcpy(bytes.data + HEADER_SIZE, key.data, key.count);
    memcpy(bytes.data + HEADER_SIZE + key.count, value.data, value.count);
    put_u32(bytes, 0, checksum(slice(bytes, 4, bytes.count - 4)));

    return bytes;
}

// Reads the record that starts at `at`. For `.BAD_CHECKSUM` the returned record has its `size`, so
// the caller can tell a damaged last record from damage in the middle of the log.
decode_record :: (log: string, at: s64) -> Record, Decode_Status {
    record: Record;
    rest := slice(log, at, log.count - at);

    if rest.count < HEADER_SIZE return record, .TORN;

    kind := rest[4];
    key_size := cast(s64) get_u32(rest, 5);
    value_size := cast(s64) get_u32(rest, 9);

    if kind != cast(u8) Record_Kind.PUT && kind != cast(u8) Record_Kind.DELETE return record, .BAD_HEADER;
    if key_size == 0 || key_size > MAX_KEY_SIZE || value_size > MAX_VALUE_SIZE return record, .BAD_HEADER;

    record.size = HEADER_SIZE + key_size + value_size;

    if rest.count < record.size return record, .TORN;

    if get_u32(rest, 0) != checksum(slice(rest, 4, record.size - 4)) return record, .BAD_CHECKSUM;

    record.kind = cast(Record_Kind) kind;
    record.key = slice(rest, HEADER_SIZE, key_size);
    record.value = slice(rest, HEADER_SIZE + key_size, value_size);

    return record, .OK;
}

put_u32 :: (bytes: string, at: s64, value: u32) {
    for 0..3 bytes[at + it] = cast,trunc(u8) (value >> (8 * it));
}

get_u32 :: (bytes: string, at: s64) -> u32 {
    value: u32;

    for 0..3 value |= (cast(u32) bytes[at + it]) << (8 * it);

    return value;
}
