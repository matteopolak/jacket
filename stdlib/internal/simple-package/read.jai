init_from_memory :: (package: *Load_Package, data: [] u8, name_for_debugging := "") -> bool {
    release_metadata(package);
    package.* = .{};
    bytes := string.{ data.count, data.data };
    valid, header := read_header(bytes, name_for_debugging);

    if !valid return false;

    offset := header.table_of_contents_offset;
    if offset < 64 || offset > bytes.count || bytes.count - offset <= 64 return fail(name_for_debugging, offset, "invalid table-of-contents offset");

    package.header = header;
    package.data = data;
    if !load_table_of_contents(package, .{ data.count - offset, data.data + offset }, data.data, name_for_debugging, offset) {
        package.* = .{};
        return false;
    }

    return true;
}

init_from_file :: (package: *Load_Package, filename: string) -> bool {
    bytes, ok := Files.read_entire_file(filename, log_errors = false);
    if !ok return fail(filename, 0, "unable to read package");

    if !init_from_memory(package, .{ bytes.count, bytes.data }, filename) {
        B.free(bytes);
        return false;
    }

    return true;
}

load_table_of_contents_only :: (filename: string, okay_if_it_does_not_exist := false) -> (success: bool, package: Load_Package) {
    package: Load_Package;
    file, opened := Files.file_open(filename, log_errors = !okay_if_it_does_not_exist);

    if !opened return false, package;

    defer Files.file_close(*file);
    length, length_ok := Files.file_length(file);
    if !length_ok || length < 64 return fail(filename, 0, "package header is truncated"), package;

    header_bytes: [64] u8;
    read_ok, amount := Files.file_read(file, header_bytes.data, 64);

    if !read_ok || amount != 64 return fail(filename, 0, "unable to read header"), package;

    valid, header := read_header(.{ 64, header_bytes.data }, filename);
    if !valid return false, package;

    offset := header.table_of_contents_offset;
    if offset < 64 || offset > length || length - offset <= 64 return fail(filename, offset, "invalid table-of-contents offset"), package;
    if !Files.file_set_position(file, offset) return fail(filename, offset, "unable to seek to table"), package;

    bytes, bytes_ok := Files.read_entire_file(file);
    if !bytes_ok || bytes.count != length - offset {
        B.free(bytes);
        return fail(filename, offset, "unable to read complete table"), package;
    }

    package.header = header;
    package.data = .{ bytes.count, bytes.data };
    if !load_table_of_contents(*package, package.data, null, filename, offset) {
        B.free(bytes);
        return false, .{};
    }

    return true, package;
}

load_table_of_contents :: (package: *Load_Package, data: [] u8, base_file_data_if_file_is_loaded: *u8, name_for_debugging: string, offset_for_debugging: s64) -> bool {
    release_metadata(package);
    bytes := string.{ data.count, data.data };
    if !Wire.contains_extent(bytes, 0, 64) || Wire.load_u32(bytes, 0) != TOC_MAGIC return fail(name_for_debugging, offset_for_debugging, "invalid or truncated table");

    toc := Table_Of_Contents.{ table_of_contents_magic = Wire.load_u32(bytes, 0), reserved0 = Wire.load_u32(bytes, 4), number_of_entries = cast,no_check(s64) Wire.load_u64(bytes, 8) };
    for *value, index: toc.reserved value.* = Wire.load_u64(bytes, 16 + 8 * index);

    count := toc.number_of_entries;
    if count < 0 || count > (bytes.count - 64) / 21 || count > 0x7fffffffffffffff / size_of(Entry_Info) return fail(name_for_debugging, offset_for_debugging, "invalid entry count");

    entries := ([] Entry_Info).{ count, cast(*Entry_Info) B.alloc(count * size_of(Entry_Info)) };
    if count > 0 && !entries.data return fail(name_for_debugging, offset_for_debugging, "entry allocation failed");

    memset(entries.data, 0, count * size_of(Entry_Info));
    package.entries = entries;
    // Header offset, not relative allocation addresses, bounds payload views.
    payload_limit := package.header.table_of_contents_offset;
    if payload_limit < 64 return malformed(package, name_for_debugging, offset_for_debugging, "invalid header payload boundary");

    cursor: s64 = 64;

    for *entry: entries {
        if !Wire.contains_extent(bytes, cursor, 4) return malformed(package, name_for_debugging, cursor, "missing name length");

        name_length := cast(s64) Wire.load_u32(bytes, cursor);

        cursor += 4;
        if !Wire.contains_extent(bytes, cursor, name_length) || !Wire.contains_extent(bytes, cursor + name_length, 17) return malformed(package, name_for_debugging, cursor, "truncated entry");
        if bytes[cursor + name_length] != 0 return malformed(package, name_for_debugging, cursor, "name is not terminated");

        entry.name = .{ name_length, bytes.data + cursor };
        cursor += name_length + 1;
        size := cast,no_check(s64) Wire.load_u64(bytes, cursor);
        offset := cast,no_check(s64) Wire.load_u64(bytes, cursor + 8);

        cursor += 16;
        if size < 0 || offset < 64 || offset > 0x7fffffffffffffff - size return malformed(package, name_for_debugging, cursor, "invalid payload extent");
        if payload_limit >= 0 && (offset > payload_limit || size > payload_limit - offset) return malformed(package, name_for_debugging, cursor, "payload overlaps table or is outside file");

        entry.offset_from_start_of_file = offset;
        entry.size_of_entry = size;

        if base_file_data_if_file_is_loaded {
            entry.data = .{ size, base_file_data_if_file_is_loaded + offset };
        }
    }

    for *entry: entries {
        if !HT.table_contains(*package.lookup, entry.name) HT.table_add(*package.lookup, entry.name, entry);
    }

    package.table_of_contents = toc;

    return true;
}

#scope_module
release_metadata :: (package: *Load_Package) {
    HT.deinit(*package.lookup);
    B.array_free(package.entries);
    package.entries = .{};
    package.table_of_contents = .{};
}

#scope_file
read_header :: (bytes: string, name: string) -> bool, Package_Header {
    if !Wire.contains_extent(bytes, 0, 64) return fail(name, 0, "package header is truncated"), .{};

    header := Package_Header.{ magic = Wire.load_u32(bytes, 0), version = Wire.load_u32(bytes, 4), flags = Wire.load_u32(bytes, 8), reserved0 = Wire.load_u32(bytes, 12), table_of_contents_offset = cast,no_check(s64) Wire.load_u64(bytes, 16) };
    for *value, index: header.reserved value.* = Wire.load_u64(bytes, 24 + 8 * index);
    if header.magic != HEADER_MAGIC || header.version > 1 return fail(name, 0, "unknown package signature or version"), .{};

    return true, header;
}

fail :: (name: string, offset: s64, message: string) -> bool {
    B.log_error("Package '%' near offset %: %", name, offset, message);
    return false;
}

malformed :: (package: *Load_Package, name: string, offset: s64, message: string) -> bool {
    release_metadata(package);
    return fail(name, offset, message);
}
