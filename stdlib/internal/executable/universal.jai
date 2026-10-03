parse_apple_universal_file :: (path: string) -> success: bool, Apple_Universal_Binary {
    content, read_ok := F.read_entire_file(path);
    if !read_ok return false, .{};

    ok, value := parse_apple_universal(content, path);
    if !ok B.free(content);

    return ok, value;
}

parse_apple_universal :: (content: string, filename: string, verbose := false) -> success: bool, Apple_Universal_Binary {
    files: [..] Arch_File;
    transferred := false;

    defer if !transferred B.array_free(files);
    if !is_apple_universal(content) || content.count < 8 return false, .{};

    big := content[0] == 0xca;
    wide := content[3] == 0xbf || content[0] == 0xbf;
    stride: u64 = 20;

    if wide stride = 32;

    count := fat_word(content.data + 4, 4, big);
    if count > (cast(u64) content.count - 8) / stride return false, .{};

    table_end := 8 + count * stride;

    for 0..cast(s64) count - 1 {
        entry := content.data + 8 + (cast(u64) it) * stride;
        cpu := cast(Cpu_Type) fat_word(entry, 4, big);
        subtype := cast(Cpu_Subtype) fat_word(entry + 4, 4, big);
        width: s64 = 4;

        if wide width = 8;

        offset := fat_word(entry + 8, width, big);
        bytes := fat_word(entry + 8 + width, width, big);
        alignment := fat_word(entry + 8 + width * 2, 4, big);

        if alignment >= 64 || offset < table_end || offset > cast(u64) content.count return false, .{};
        if bytes > cast(u64) content.count - offset return false, .{};

        unit: u64 = 1;
        unit <<= alignment;
        if offset % unit != 0 return false, .{};
        if wide && fat_word(entry + 28, 4, big) != 0 return false, .{};

        // Pairwise validation is bounded by the already checked table count.
        for previous: files {
            previous_offset := cast(u64) (previous.data.data - content.data);
            previous_end := previous_offset + cast(u64) previous.data.count;
            if bytes != 0 && previous.data.count != 0 && offset < previous_end && previous_offset < offset + bytes return false, .{};
            if cpu == previous.cpu_type && subtype == previous.cpu_subtype return false, .{};
        }

        record: Arch_File;
        record.cpu_type = cpu;
        record.cpu_subtype = subtype;
        record.data = .{ cast(s64) bytes, content.data + offset };
        B.array_add(*files, record);
    }

    result: Apple_Universal_Binary;
    result.content = content;
    result.files = .{ files.count, files.data };
    transferred = true;

    return true, result;
}

#scope_file
B :: #import "Basic";
F :: #import "File";

fat_word :: (data: *u8, bytes: s64, big: bool) -> u64 {
    result: u64;

    for 0..bytes - 1 {
        index := it;
        if !big index = bytes - it - 1;
        lane := cast(u64) data[index];
        result = (result << 8) | lane;
    }

    return result;
}
