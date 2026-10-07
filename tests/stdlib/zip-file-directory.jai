// Zip_File_Directory reads the central directory of a classic ZIP archive; deinit releases the
// directory storage and leaves the caller's file handle alone.
#import "Basic";
#import "File";
Zip :: #import "Zip_File_Directory";

put16 :: (b: *String_Builder, v: int) {
    append(b, cast(u8) (v & 0xff));
    append(b, cast(u8) ((v >> 8) & 0xff));
}

put32 :: (b: *String_Builder, v: int) {
    put16(b, v & 0xffff);
    put16(b, (v >> 16) & 0xffff);
}

// Two stored entries, with an archive comment after the end record.
build_archive :: () -> string {
    names :: string.["a.txt", "dir/bee.bin"];
    payloads :: string.["hi", "buzzing"];
    b: String_Builder;
    offsets: [2] int;
    for names {
        offsets[it_index] = builder_string_length(*b);
        put32(*b, 0x04034b50);
        put16(*b, 20);
        put16(*b, 0);
        put16(*b, 0);
        put16(*b, 0);
        put16(*b, 0);
        put32(*b, 0x1234_5678 + it_index);
        put32(*b, payloads[it_index].count);
        put32(*b, payloads[it_index].count);
        put16(*b, it.count);
        put16(*b, 3); // a local extra field the central directory does not mention
        append(*b, it);
        append(*b, "xyz");
        append(*b, payloads[it_index]);
    }
    directory_start := builder_string_length(*b);
    for names {
        put32(*b, 0x02014b50);
        put16(*b, 0x31e);
        put16(*b, 20);
        put16(*b, 0);
        put16(*b, 0);
        put16(*b, 0);
        put16(*b, 0);
        put32(*b, 0x1234_5678 + it_index);
        put32(*b, payloads[it_index].count);
        put32(*b, payloads[it_index].count);
        put16(*b, it.count);
        put16(*b, 0);
        put16(*b, 0);
        put16(*b, 0);
        put16(*b, 1);
        put32(*b, 0x81a4_0000);
        put32(*b, offsets[it_index]);
        append(*b, it);
    }
    directory_size := builder_string_length(*b) - directory_start;
    put32(*b, 0x06054b50);
    put16(*b, 0);
    put16(*b, 0);
    put16(*b, 2);
    put16(*b, 2);
    put32(*b, directory_size);
    put32(*b, directory_start);
    comment :: "archive comment";
    put16(*b, comment.count);
    append(*b, comment);
    return builder_to_string(*b);
}

main :: () -> s64 {
    path := "/tmp/jaic_zip_file_directory_test.zip";
    if !write_entire_file(path, build_archive()) return 1;
    defer file_delete(path);

    ok, dir := Zip.load_zip_directory(path);
    if !ok || dir.entries.count != 2 || dir.name != path return 2;
    first := dir.entries[0];
    second := dir.entries[1];
    if first.filename != "a.txt" || second.filename != "dir/bee.bin" return 3;
    if first.crc != 0x1234_5678 || second.crc != 0x1234_5679 return 4;
    if first.compressed_len != 2 || second.chunk_len != 7 return 5;
    if first.version_made_by != 0x31e || first.internal_attributes != 1 || first.external_attributes != 0x81a4_0000 return 6;
    // Data starts after the 30-byte local header, the name and the local extra field.
    if first.chunk_offset != 30 + 5 + 3 return 7;
    if second.chunk_offset != 30 + 5 + 3 + 2 + 30 + 11 + 3 return 8;
    Zip.deinit(*dir);
    if dir.entries.count != 0 || dir.entries.data != null || dir.memory_block.count != 0 || dir.name.count != 0 return 9;

    // Keeping the file open: deinit must not touch the handle.
    ok, dir = Zip.load_zip_directory(path, close_file = false);
    if !ok return 10;
    handle := dir.file;
    Zip.deinit(*dir);
    if dir.file.handle != handle.handle return 11;
    file_close(*dir.file);

    // Not an archive.
    if !write_entire_file(path, "plain text, no end record here at all") return 12;
    bad := Zip.load_zip_directory(path);
    if bad return 13;
    return 0;
}
