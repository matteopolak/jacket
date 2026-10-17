#import "Basic";
P :: #import "Simple_Package";
F :: #import "File";
H :: #import "Hash_Table";

errors: int;

capture :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0 && data == null && (info.common_flags & .ERROR) != 0);
    errors += 1;
}

release_metadata :: (package: *P.Load_Package) {
    H.deinit(*package.lookup);
    array_free(package.entries);
    package.* = .{};
}

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 2);
    path := sprint("%/package.simp", args[1]);

    defer free(path);
    creator: P.Create_Package;

    P.init(*creator);
    defer P.deinit(*creator);
    name := copy_string("zeta");
    payload: [3] u8 = .[1, 2, 3];

    P.add(*creator, name, payload);
    assert(creator.entries[0].name.data != name.data && creator.entries[0].data.data == payload.data);
    name.data[0] = 88;
    free(name);
    payload[1] = 99;
    P.add(*creator, "alpha", .{});
    assert(P.write(*creator, path));
    bytes, read_ok := F.read_entire_file(path);

    assert(read_ok && bytes.count == 182);
    defer free(bytes);
    loaded: P.Load_Package;

    assert(P.init_from_memory(*loaded, .{ bytes.count, bytes.data }, "memory"));
    assert(loaded.header.magic == 0x706d6973 && loaded.header.version == 1);
    assert(loaded.header.table_of_contents_offset == 67 && loaded.table_of_contents.number_of_entries == 2);
    assert(loaded.entries[0].name == "alpha" && loaded.entries[1].name == "zeta");
    assert(loaded.entries[1].data.data == bytes.data + 64 && loaded.entries[1].data[1] == 99);
    found, entry := H.table_find(*loaded.lookup, "zeta");

    assert(found && entry == *loaded.entries[1]);
    header_for_direct := loaded.header;

    release_metadata(*loaded);
    loaded.header = header_for_direct;
    assert(P.load_table_of_contents(*loaded, .{ bytes.count - 67, bytes.data + 67 }, bytes.data, "direct", 67));
    assert(loaded.entries[1].data.data == bytes.data + 64 && loaded.entries[1].data[1] == 99);
    release_metadata(*loaded);
    toc_copy := copy_string(.{ bytes.count - 67, bytes.data + 67 });

    loaded.header = header_for_direct;
    assert(P.load_table_of_contents(*loaded, .{ toc_copy.count, toc_copy.data }, bytes.data, "separate", 999));
    assert(loaded.entries[1].name.data >= toc_copy.data && loaded.entries[1].name.data < toc_copy.data + toc_copy.count);
    assert(loaded.entries[1].data.data == bytes.data + 64);
    release_metadata(*loaded);
    free(toc_copy);
    assert(P.init_from_file(*loaded, path));
    owned := loaded.data.data;

    assert(loaded.entries[1].data[1] == 99);
    release_metadata(*loaded);
    free(owned);
    toc_ok, toc := P.load_table_of_contents_only(path);

    assert(toc_ok && toc.entries.count == 2 && toc.entries[1].name == "zeta");
    assert(toc.entries[1].data.count == 0 && toc.entries[1].data.data == null);
    owned = toc.data.data;
    release_metadata(*toc);
    free(owned);
    assert(P.write(*creator, path, sort_contents = false));
    assert(P.init_from_file(*loaded, path));
    assert(loaded.entries[0].name == "zeta" && loaded.entries[1].name == "alpha");
    owned = loaded.data.data;
    release_metadata(*loaded);
    free(owned);

    unicode_creator: P.Create_Package;

    P.init(*unicode_creator);
    defer P.deinit(*unicode_creator);
    P.add(*unicode_creator, "café/字", payload);
    unicode_path := sprint("%/unicode.simp", args[1]);

    defer free(unicode_path);
    assert(P.write(*unicode_creator, unicode_path));
    assert(P.init_from_file(*loaded, unicode_path));
    assert(loaded.entries[0].name == "café/字" && loaded.entries[0].data[1] == 99);
    owned = loaded.data.data;
    release_metadata(*loaded);
    free(owned);
    duplicate_creator: P.Create_Package;

    P.init(*duplicate_creator);
    defer P.deinit(*duplicate_creator);
    first_payload: [1] u8 = .[10];
    second_payload: [1] u8 = .[20];

    P.add(*duplicate_creator, "same", first_payload);
    P.add(*duplicate_creator, "same", second_payload);
    duplicate_path := sprint("%/duplicates.simp", args[1]);

    defer free(duplicate_path);
    assert(P.write(*duplicate_creator, duplicate_path));
    assert(P.init_from_file(*loaded, duplicate_path));
    first_found, first_entry := H.table_find(*loaded.lookup, "same");

    assert(first_found && loaded.entries.count == 2 && first_entry == *loaded.entries[0]);
    assert(first_entry.data[0] == 10 && loaded.entries[1].data[0] == 20);
    owned = loaded.data.data;
    release_metadata(*loaded);
    free(owned);
    empty_creator: P.Create_Package;

    P.init(*empty_creator);
    defer P.deinit(*empty_creator);
    empty_path := sprint("%/empty.simp", args[1]);

    defer free(empty_path);
    assert(P.write(*empty_creator, empty_path));
    empty_bytes, empty_read := F.read_entire_file(empty_path);

    assert(empty_read && empty_bytes.count == 128);
    defer free(empty_bytes);
    // Accepted version zero and opaque flags/reserved fields.
    bytes.data[4] = 0;
    bytes.data[8] = 255;
    bytes.data[24] = 255;
    assert(P.init_from_memory(*loaded, .{ bytes.count, bytes.data }));
    assert(loaded.header.version == 0 && loaded.header.flags == 255 && loaded.header.reserved[0] == 255);
    release_metadata(*loaded);
    bytes.data[4] = 1;

    push_context {
        context.logger = capture;
        context.logger_data = null;

        missing := sprint("%/absent-package.simp", args[1]);
        optional_ok, optional := P.load_table_of_contents_only(missing, okay_if_it_does_not_exist = true);

        free(missing);
        assert(!optional_ok && optional.data.data == null && errors == 0);

        for prefix: 0..bytes.count - 1 {
            assert(!P.init_from_memory(*loaded, .{ prefix, bytes.data }));
            assert(loaded.entries.count == 0 && loaded.data.data == null);
        }

        assert(!P.init_from_memory(*loaded, .{ empty_bytes.count, empty_bytes.data }));
        // Huge entry count, huge name length and signed negative payload extent.
        original_count := bytes.data[67 + 8];

        bytes.data[67 + 8] = 255;
        assert(!P.init_from_memory(*loaded, .{ bytes.count, bytes.data }));
        bytes.data[67 + 8] = original_count;
        for index: 0..3 bytes.data[67 + 64 + index] = 255;

        assert(!P.init_from_memory(*loaded, .{ bytes.count, bytes.data }));
        bytes.data[67 + 64] = 5;
        for index: 1..3 bytes.data[67 + 64 + index] = 0;

        original_offset := bytes.data[67 + 64 + 18];

        bytes.data[67 + 64 + 18] = 68;
        assert(!P.init_from_memory(*loaded, .{ bytes.count, bytes.data }));
        malformed_path := sprint("%/overlap.simp", args[1]);

        assert(F.write_entire_file(malformed_path, bytes));
        toc_bad, rejected := P.load_table_of_contents_only(malformed_path);

        free(malformed_path);
        assert(!toc_bad && rejected.entries.count == 0);
        bytes.data[67 + 64 + 18] = original_offset;
        // alpha's size follows u32 name length, 5 name bytes and NUL.
        for index: 0..7 bytes.data[67 + 64 + 10 + index] = 255;

        assert(!P.init_from_memory(*loaded, .{ bytes.count, bytes.data }));
        for index: 0..7 bytes.data[67 + 64 + 10 + index] = 0;

        bytes.data[4] = 2;
        assert(!P.init_from_memory(*loaded, .{ bytes.count, bytes.data }));
        bytes.data[4] = 1;
        bytes.data[67 + 64 + 4 + 5] = 1;
        assert(!P.init_from_memory(*loaded, .{ bytes.count, bytes.data }));
        assert(loaded.entries.count == 0 && loaded.lookup.count == 0 && loaded.data.count == 0);
    }

    assert(errors == bytes.count + 8);
    assert(size_of(P.Package_Header) == 64 && size_of(P.Table_Of_Contents) == 64);
    assert(type_info(P.Entry_Info).name == "Entry_Info");
    print("ok\n");
}
