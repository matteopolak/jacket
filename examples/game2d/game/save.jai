// The save file: a small header, an LZ4-compressed body and an xxHash64 of everything before the
// checksum, so a truncated or edited file is rejected rather than half read.
//
//     "G2SV" | version u32 | raw size u32 | compressed size u32 | compressed body | xxh64 u64
//
// The body is the fields of `Save_Data` in order: integers as little-endian s64, strings as a u32
// length and the bytes.

SAVE_MAGIC :: "G2SV";
SAVE_VERSION :: 1;
RECENT_SCORES :: 8;

Save_Data :: struct {
    high_score: s64;
    games: s64;
    account_id: s64;
    name: string;
    token: string;
    recent: [..] s64;
    muted: bool;
}

save_free :: (save: *Save_Data) {
    free(save.name);
    free(save.token);
    array_free(save.recent);

    save.* = .{};
}

remember_score :: (save: *Save_Data, score: s64) {
    save.games += 1;
    save.high_score = max(save.high_score, score);

    array_add(*save.recent, score);

    if save.recent.count > RECENT_SCORES array_ordered_remove_by_index(*save.recent, 0);
}

put_s64 :: (out: *String_Builder, value: s64) {
    append(out, string.{ 8, cast(*u8) *value });
}

put_text :: (out: *String_Builder, text: string) {
    length := cast(u32) text.count;

    append(out, string.{ 4, cast(*u8) *length });
    append(out, text);
}

// The bytes of the file for `save`.
save_encode :: (save: Save_Data) -> string {
    body: String_Builder;

    put_s64(*body, save.high_score);
    put_s64(*body, save.games);
    put_s64(*body, save.account_id);
    put_s64(*body, ifx save.muted then 1 else 0);
    put_text(*body, save.name);
    put_text(*body, save.token);
    put_s64(*body, save.recent.count);

    for save.recent put_s64(*body, it);

    raw := builder_to_string(*body);

    defer free(raw);

    capacity := LZ4_compressBound(cast(s32) raw.count);
    packed := alloc_string(capacity);

    defer free(packed);

    packed_size := LZ4_compress_default(raw.data, packed.data, cast(s32) raw.count, capacity);

    out: String_Builder;

    append(*out, SAVE_MAGIC);
    put_u32(*out, SAVE_VERSION);
    put_u32(*out, cast(u32) raw.count);
    put_u32(*out, cast(u32) packed_size);
    append(*out, string.{ packed_size, packed.data });

    unchecked := builder_to_string(*out);

    digest := XXH64_of(unchecked);

    complete: String_Builder;

    append(*complete, unchecked);
    append(*complete, string.{ 8, cast(*u8) *digest });
    free(unchecked);

    return builder_to_string(*complete);
}

put_u32 :: (out: *String_Builder, value: u32) {
    append(out, string.{ 4, cast(*u8) *value });
}

XXH64_of :: (bytes: string) -> u64 {
    state: XXH64_state_t;

    XXH64_reset(*state, 0);
    XXH64_update(*state, bytes.data, cast(u32) bytes.count);

    return XXH64_digest(*state);
}

Reader :: struct {
    bytes: string;
    at: s64;
    failed: bool;
}

take :: (reader: *Reader, count: s64) -> string {
    if reader.failed || count < 0 || reader.at + count > reader.bytes.count {
        reader.failed = true;

        return "";
    }

    result := string.{ count, reader.bytes.data + reader.at };

    reader.at += count;

    return result;
}

take_s64 :: (reader: *Reader) -> s64 {
    bytes := take(reader, 8);

    if bytes.count != 8 return 0;

    return (cast(*s64) bytes.data).*;
}

take_text :: (reader: *Reader) -> string {
    prefix := take(reader, 4);

    if prefix.count != 4 return "";

    return copy_string(take(reader, cast(s64) (cast(*u32) prefix.data).*));
}

// Parses a save file; `why` says what is wrong with one that does not load.
save_decode :: (bytes: string) -> Save_Data, bool, string {
    header :: 16;
    checksum :: 8;

    if bytes.count < header + checksum || !begins_with(bytes, SAVE_MAGIC) return .{}, false, "not a save file";

    body_end := bytes.count - checksum;
    expected := (cast(*u64) (bytes.data + body_end)).*;

    if XXH64_of(.{ body_end, bytes.data }) != expected return .{}, false, "checksum mismatch";

    version := (cast(*u32) (bytes.data + 4)).*;
    raw_size := cast(s64) (cast(*u32) (bytes.data + 8)).*;
    packed_size := cast(s64) (cast(*u32) (bytes.data + 12)).*;

    if version != SAVE_VERSION return .{}, false, "a save from another version";
    if header + packed_size != body_end || raw_size > 1 << 20 return .{}, false, "bad sizes";

    raw := alloc_string(raw_size);

    defer free(raw);

    got := LZ4_decompress_safe(bytes.data + header, raw.data, cast(s32) packed_size, cast(s32) raw_size);

    if got != raw_size return .{}, false, "the body does not decompress";

    reader := Reader.{ raw, 0, false };
    save: Save_Data;

    save.high_score = take_s64(*reader);
    save.games = take_s64(*reader);
    save.account_id = take_s64(*reader);
    save.muted = take_s64(*reader) != 0;
    save.name = take_text(*reader);
    save.token = take_text(*reader);

    count := take_s64(*reader);

    if count < 0 || count > 1000 reader.failed = true;

    for 1..count array_add(*save.recent, take_s64(*reader));

    if reader.failed {
        save_free(*save);

        return .{}, false, "the body is cut short";
    }

    return save, true, "";
}

save_to_file :: (path: string, save: Save_Data) -> bool {
    bytes := save_encode(save);

    // Write beside the file and rename, so a crash never leaves half a save.
    temporary := tprint("%.tmp", path);

    if !write_entire_file(temporary, bytes) return false;

    return file_move(temporary, path);
}

save_from_file :: (path: string) -> Save_Data, bool, string {
    if !file_exists(path) return .{}, false, "no save yet";

    bytes, ok := read_entire_file(path);

    if !ok return .{}, false, "cannot read it";

    defer free(bytes);

    return save_decode(bytes);
}
