// Native notification views. No allocation; views live only as long as input.
Record :: struct {
    descriptor: s32;
    flags: u32;
    cookie: u32;
    name: [] u8;
}

inotify_record :: (data: [] u8, cursor: s64) -> Record, next: s64, valid: bool {
    if !range_valid(data, cursor, 16) return .{}, cursor, false;

    length := read_u32(data, cursor + INOTIFY_NAME_LENGTH_OFFSET);
    if cast(s64) length > data.count - cursor - 16 return .{}, cursor, false;

    result := Record.{ descriptor = cast,trunc(s32) read_u32(data, cursor), flags = read_u32(data, cursor + 4), cookie = read_u32(data, cursor + 8) };

    if length > 0 {
        size: s64;
        while size < length && data[cursor + 16 + size] != 0 size += 1;
        if size == length return .{}, cursor, false;
        result.name = .{ size, data.data + cursor + 16 };
    }

    return result, cursor + 16 + length, true;
}

windows_record :: (data: [] u8, cursor: s64) -> Record, next: s64, valid: bool {
    if !range_valid(data, cursor, 12) return .{}, cursor, false;

    offset := read_u32(data, cursor);
    length := read_u32(data, cursor + 8);

    if length % 2 != 0 || cast(s64) length > data.count - cursor - WINDOWS_RECORD_HEADER_SIZE return .{}, cursor, false;
    if offset != 0 && (offset % 4 != 0 || cast(s64) offset < WINDOWS_RECORD_HEADER_SIZE + cast(s64) length || cast(s64) offset > data.count - cursor - WINDOWS_RECORD_HEADER_SIZE) return .{}, cursor, false;

    result := Record.{ flags = read_u32(data, cursor + 4), name = .{ cast(s64) length, data.data + cursor + WINDOWS_RECORD_HEADER_SIZE } };

    return result, ifx offset then cursor + offset else data.count, true;
}

#scope_file

INOTIFY_NAME_LENGTH_OFFSET :: 12;
WINDOWS_RECORD_HEADER_SIZE :: 12;

range_valid :: (data: [] u8, cursor: s64, bytes: s64) -> bool {
    return data.count >= 0 && cursor >= 0 && cursor <= data.count && bytes <= data.count - cursor && (data.count == 0 || data.data != null);
}

read_u32 :: (data: [] u8, cursor: s64) -> u32 {
    a := cast(u32) data[cursor];
    b := cast(u32) data[cursor + 1];
    c := cast(u32) data[cursor + 2];
    d := cast(u32) data[cursor + 3];

    return a | (b << 8) | (c << 16) | (d << 24);
}
