deinit :: (dir: *Zip_Directory) {
    free(dir.name);
    array_free(dir.entries);
    free(dir.memory_block);
    // The public contract deliberately leaves any retained file to its caller.
    retained := dir.file;

    dir.* = .{};
    dir.file = retained;
} @FreesArgument

load_zip_directory :: (zip_filename: string, close_file := true) -> (success: bool, dir: Zip_Directory) {
    file, opened := file_open(zip_filename, log_errors = false);
    if !opened return false, .{};

    result: Zip_Directory;
    result.file = file;
    completed := false;

    defer {
        if !completed {
            deinit(*result);
            file_close(*file);
        }
    }

    length, sized := file_length(file);
    if !sized || length < END_RECORD_SIZE return false, .{};

    tail_length := min(length, cast(s64) (END_RECORD_SIZE + LONGEST_COMMENT));
    if !file_set_position(file, length - tail_length) return false, .{};

    tail := alloc_string(tail_length);

    defer free(tail);
    read_ok, received := file_read(file, tail.data, tail.count);
    if !read_ok || received != tail.count return false, .{};

    end_offset: s64 = -1;
    candidate := tail.count - END_RECORD_SIZE;

    while candidate >= 0 {
        bytes := tail.data + candidate;
        if read_u32(bytes) == 0x06054b50 && candidate + END_RECORD_SIZE + read_u16(bytes + 20) == tail.count {
            end_offset = candidate;
            break;
        }

        candidate -= 1;
    }

    if end_offset < 0 return false, .{};

    end := tail.data + end_offset;
    entry_count := cast(s64) read_u16(end + 10);
    directory_size := cast(s64) read_u32(end + 12);
    directory_start := cast(s64) read_u32(end + 16);

    if read_u16(end + 4) != 0 || read_u16(end + 6) != 0 || read_u16(end + 8) != entry_count return false, .{};
    if entry_count == 0xffff || directory_size == 0xffffffff || directory_start == 0xffffffff return false, .{};

    end_position := length - tail_length + end_offset;
    if directory_start > end_position || directory_size > end_position - directory_start return false, .{};
    if entry_count > directory_size / CENTRAL_HEADER_SIZE return false, .{};
    if !file_set_position(file, directory_start) return false, .{};

    result.memory_block = alloc_string(directory_size);

    if directory_size > 0 {
        read_ok, received = file_read(file, result.memory_block.data, directory_size);
        if !read_ok || received != directory_size return false, .{};
    }

    cursor: s64;

    for 0..entry_count - 1 {
        if cursor > directory_size - CENTRAL_HEADER_SIZE return false, .{};

        bytes := result.memory_block.data + cursor;

        if read_u32(bytes) != 0x02014b50 return false, .{};

        entry: Zip_Entry;
        entry.position_in_file = directory_start + cursor;
        entry.version_made_by = read_u16(bytes + 4);
        entry.version_needed_to_extract = read_u16(bytes + 6);
        entry.bit_flag = read_u16(bytes + 8);
        if entry.bit_flag & 8 != 0 return false, .{};

        entry.crc = read_u32(bytes + 16);
        entry.compressed_len = read_u32(bytes + 20);
        entry.chunk_len = read_u32(bytes + 24);
        entry.name_len = read_u16(bytes + 28);
        entry.extra_field_len = read_u16(bytes + 30);
        entry.file_comment_len = read_u16(bytes + 32);
        if read_u16(bytes + 34) != 0 return false, .{};

        entry.internal_attributes = read_u16(bytes + 36);
        entry.external_attributes = read_u32(bytes + 38);
        entry.chunk_offset = read_u32(bytes + 42);
        if entry.compressed_len == 0xffffffff || entry.chunk_len == 0xffffffff || entry.chunk_offset == 0xffffffff return false, .{};

        record_size := cast(s64) CENTRAL_HEADER_SIZE + entry.name_len + entry.extra_field_len + entry.file_comment_len;
        if record_size > directory_size - cursor return false, .{};

        entry.filename = .{ entry.name_len, bytes + CENTRAL_HEADER_SIZE };
        extra: string = .{ entry.extra_field_len, bytes + CENTRAL_HEADER_SIZE + entry.name_len };
        if !classic_extra_fields(extra) return false, .{};

        array_add(*result.entries, entry);
        cursor += record_size;
    }

    if cursor != directory_size return false, .{};

    result.name = copy_string(zip_filename);
    // Close before copying the returned aggregate; deferred mutation would only
    // alter the local copy of its file field.
    if close_file file_close(*result.file);

    completed = true;

    return true, result;
}

#scope_file
#import "Basic";
#import "File";

// Fixed parts of the end-of-central-directory record and of a central directory entry.
END_RECORD_SIZE :: 22;
CENTRAL_HEADER_SIZE :: 46;
LONGEST_COMMENT :: 0xffff;

classic_extra_fields :: (extra: string) -> bool {
    cursor: s64;
    while cursor < extra.count {
        if extra.count - cursor < 4 return false;

        id := read_u16(extra.data + cursor);
        size := cast(s64) read_u16(extra.data + cursor + 2);

        cursor += 4;
        if id == 1 || size > extra.count - cursor return false;

        cursor += size;
    }

    return true;
}

read_u16 :: (bytes: *u8) -> u16 {
    low := cast(u16) bytes[0];
    high := cast(u16) bytes[1];
    return low | (high << 8);
}

read_u32 :: (bytes: *u8) -> u32 {
    a := cast(u32) bytes[0];
    b := cast(u32) bytes[1];
    c := cast(u32) bytes[2];
    d := cast(u32) bytes[3];

    return a | (b << 8) | (c << 16) | (d << 24);
}
