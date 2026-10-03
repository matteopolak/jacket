Archive_File :: struct {
    name: string;
    header_start_offset: u32;
    data: [] u8;
}

Archive :: struct {
    content: string;
    symbol_file_offsets: [] u32;
    symbol_names: [] string;
    files: [] Archive_File;
}

is_ar :: (file_content: string) -> bool {
    return file_content.count >= 8 && part(file_content, 0, 8) == "!<arch>\n";
}

is_apple_universal :: (file_content: string) -> bool {
    if file_content.count < 4 return false;
    magic := word(file_content.data, true);
    return magic == 0xcafebabe || magic == 0xbebafeca || magic == 0xcafebabf || magic == 0xbfbafeca;
}

parse_ar_file :: (path: string) -> success: bool, Archive {
    content, read_ok := F.read_entire_file(path);
    if !read_ok return false, .{};

    ok, archive := parse_ar(content, path);
    if !ok B.free(content);

    return ok, archive;
}

parse_ar :: (content: string, archive_filename: string, verbose := false) -> success: bool, Archive {
    records: [..] Archive_File;
    offsets: [..] u32;
    names: [..] string;

    defer B.array_free(records);
    defer B.array_free(offsets);
    defer B.array_free(names);
    if !is_ar(content) return false, .{};

    longnames: string;
    first_index: string;
    second_index: string;
    bsd_index: string;
    bsd_present := false;
    first_present := false;
    second_present := false;
    cursor: s64 = 8;

    while cursor < content.count {
        if content.count - cursor < 60 || cursor > 0xffffffff return false, .{};

        header := part(content, cursor, 60);
        if header[58] != 0x60 || header[59] != 10 return false, .{};

        size_ok, bytes := decimal(part(header, 48, 10));
        if !size_ok || bytes > content.count - cursor - 60 return false, .{};

        name := trim_padding(part(header, 0, 16));
        data := part(content, cursor + 60, bytes);

        if name.count >= 3 && part(name, 0, 3) == "#1/" {
            name_ok, length := decimal(part(name, 3, name.count - 3));
            if !name_ok || length > data.count return false, .{};

            name = part(data, 0, length);
            while name.count > 0 && name[name.count - 1] == 0 name.count -= 1;

            data = part(data, length, data.count - length);
        }

        if name == "//" {
            if longnames.data return false, .{};
            longnames = data;
        } else if name == "/" {
            if !first_present {
                first_index = data;
                first_present = true;
            } else if !second_present {
                second_index = data;
                second_present = true;
            } else return false, .{};
        } else if name == "__.SYMDEF" || name == "__.SYMDEF SORTED" {
            if bsd_present return false, .{};
            bsd_index = data;
            bsd_present = true;
        } else if name == "/SYM64/" || name == "__.SYMDEF_64" || name == "__.SYMDEF_64 SORTED" {
            return false, .{};
        } else {
            if name.count == 0 return false, .{};

            record: Archive_File;
            record.name = name;
            record.header_start_offset = cast(u32) cursor;
            record.data = .{ data.count, data.data };
            B.array_add(*records, record);
        }

        cursor += 60 + bytes;
        if (cursor & 1) != 0 {
            if cursor >= content.count || content[cursor] != 10 return false, .{};
            cursor += 1;
        }
    }

    for *record: records {
        name := record.name;
        if name[0] == #char "/" {
            valid, offset := decimal(part(name, 1, name.count - 1));
            if !valid || offset >= longnames.count return false, .{};
            // References must identify starts, never substrings of a preceding name.
            if offset > 0 && longnames[offset - 1] != 0 && longnames[offset - 1] != 10 return false, .{};

            end := offset;
            while end < longnames.count && longnames[end] != 0 && longnames[end] != 10 end += 1;
            if end == longnames.count return false, .{};

            name = part(longnames, offset, end - offset);
        }

        if name.count > 0 && name[name.count - 1] == #char "/" name.count -= 1;
        if name.count == 0 return false, .{};

        record.name = name;
    }

    if bsd_present && (first_present || second_present) return false, .{};

    if bsd_present {
        // The BSD format carries no endian tag. Try both bounded interpretations.
        valid := parse_bsd_index(bsd_index, false, *offsets, *names);
        if !valid {
            offsets.count = 0;
            names.count = 0;
            valid = parse_bsd_index(bsd_index, true, *offsets, *names);
        }

        if !valid return false, .{};
    } else if second_present {
        if !parse_coff_index(second_index, *offsets, *names) return false, .{};

        // Validate the redundant first index too; never silently accept corruption.
        check_offsets: [..] u32;
        check_names: [..] string;

        defer B.array_free(check_offsets);
        defer B.array_free(check_names);
        if !parse_first_index(first_index, *check_offsets, *check_names) return false, .{};
        if !valid_offsets(check_offsets, records) return false, .{};
    } else if first_present {
        if !parse_first_index(first_index, *offsets, *names) return false, .{};
    }

    if !valid_offsets(offsets, records) return false, .{};

    result: Archive;
    result.content = content;
    result.files = B.array_copy(cast([] Archive_File) records);
    result.symbol_file_offsets = B.array_copy(cast([] u32) offsets);
    result.symbol_names = B.array_copy(cast([] string) names);
    if verbose B.print("Parsed archive %: % files, % symbols\n", archive_filename, records.count, names.count);

    return true, result;
}

#scope_file
B :: #import "Basic";
F :: #import "File";

part :: (input: string, offset: s64, count: s64) -> string {
    B.assert(offset >= 0 && count >= 0 && offset <= input.count && count <= input.count - offset);
    return .{ count, input.data + offset };
}

trim_padding :: (input: string) -> string {
    result := input;
    while result.count > 0 && result[result.count - 1] == 32 result.count -= 1;
    return result;
}

decimal :: (input: string) -> bool, s64 {
    text := trim_padding(input);
    if text.count == 0 return false, 0;

    result: s64;

    for digit: text {
        if digit < #char "0" || digit > #char "9" return false, 0;
        value := cast(s64) digit - #char "0";
        if result > (0x7fffffffffffffff - value) / 10 return false, 0;
        result = result * 10 + value;
    }

    return true, result;
}

word :: (data: *u8, big: bool) -> u32 {
    result: u32;

    for 0..3 {
        index := it;
        if !big index = 3 - it;
        lane := cast(u32) data[index];
        result = (result << 8) | lane;
    }

    return result;
}

symbol_name :: (data: string, offset: s64) -> bool, string, s64 {
    if offset < 0 || offset >= data.count return false, "", offset;

    end := offset;
    while end < data.count && data[end] != 0 end += 1;
    if end == data.count || end == offset return false, "", offset;

    return true, part(data, offset, end - offset), end + 1;
}

parse_first_index :: (data: string, offsets: *[..] u32, names: *[..] string) -> bool {
    if data.count < 4 return false;

    count := cast(s64) word(data.data, true);
    if count > (data.count - 4) / 4 return false;

    cursor := 4 + 4 * count;

    for 0..count - 1 {
        valid, name, next := symbol_name(data, cursor);
        if !valid return false;

        B.array_add(offsets, word(data.data + 4 + 4 * it, true));
        B.array_add(names, name);
        cursor = next;
    }

    // Native archives may pad index storage with zero bytes.
    while cursor < data.count {
        if data[cursor] != 0 return false;
        cursor += 1;
    }

    return true;
}

parse_coff_index :: (data: string, offsets: *[..] u32, names: *[..] string) -> bool {
    if data.count < 8 return false;

    members := cast(s64) word(data.data, false);
    if members > (data.count - 8) / 4 return false;

    count_at := 4 + 4 * members;
    count := cast(s64) word(data.data + count_at, false);

    if count > (data.count - count_at - 4) / 2 return false;

    indices := count_at + 4;
    cursor := indices + 2 * count;

    for 0..count - 1 {
        low := cast(u32) data[indices + 2 * it];
        high := cast(u32) data[indices + 2 * it + 1];
        index := cast(s64) (low | (high << 8));

        if index == 0 || index > members return false;

        valid, name, next := symbol_name(data, cursor);
        if !valid return false;

        B.array_add(offsets, word(data.data + 4 + 4 * (index - 1), false));
        B.array_add(names, name);
        cursor = next;
    }

    while cursor < data.count {
        if data[cursor] != 0 return false;
        cursor += 1;
    }

    return true;
}

parse_bsd_index :: (data: string, big: bool, offsets: *[..] u32, names: *[..] string) -> bool {
    if data.count < 8 return false;

    bytes := cast(s64) word(data.data, big);
    if (bytes & 7) != 0 || bytes > data.count - 8 return false;

    string_bytes := cast(s64) word(data.data + 4 + bytes, big);
    if string_bytes > data.count - 8 - bytes return false;

    strings := part(data, 8 + bytes, string_bytes);

    for 0..bytes / 8 - 1 {
        index := cast(s64) word(data.data + 4 + 8 * it, big);
        if index > 0 && index < strings.count && strings[index - 1] != 0 return false;

        valid, name := symbol_name(strings, index);
        if !valid return false;

        B.array_add(offsets, word(data.data + 8 + 8 * it, big));
        B.array_add(names, name);
    }

    cursor := 8 + bytes + string_bytes;
    while cursor < data.count {
        if data[cursor] != 0 return false;
        cursor += 1;
    }

    return true;
}

valid_offsets :: (offsets: [] u32, records: [] Archive_File) -> bool {
    for offset: offsets {
        lower: s64;
        upper := records.count;

        while lower < upper {
            middle := lower + (upper - lower) / 2;
            if records[middle].header_start_offset < offset lower = middle + 1;
            else upper = middle;
        }

        if lower >= records.count || records[lower].header_start_offset != offset return false;
    }

    return true;
}
