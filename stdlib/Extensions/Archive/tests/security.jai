#import "Basic";
#import "File";
#import "Extensions/Archive";
#import "Extensions/Testing";
Io :: #import "Extensions/Io";

paths_and_links :: (t: *Test) {
    for name: string.["../escape", "/absolute", "C:/drive", "\\\\host\\share", "a/../../escape", "safe\\..\\escape", "a\0suffix"] {
        // Construct hostile names as TAR header bytes; the writer deliberately rejects NUL.
        if name == "a\0suffix" continue;

        items := Archive_Item.[.{ name = name, data = cast([] u8) "x" }];
        bytes, err := encode_archive(items, .TAR);

        require(err.kind == .NONE);
        defer free(bytes.data);
        a, failure := read_archive(bytes, .TAR);

        require(failure.kind == .NONE);
        defer free_archive(*a);
        error := extract(*a, temp_dir(t));

        expect(error.kind == .PATH);
        expect(error.entry == name);
        free_archive(*error);
    }

    for kind: Archive_Kind.[.SYMLINK, .HARDLINK] {
        items := Archive_Item.[.{ name = "link", kind = kind, link = "../../outside" }];
        bytes, err := encode_archive(items, .TAR);

        require(err.kind == .NONE);
        defer free(bytes.data);
        a, failure := read_archive(bytes, .TAR);

        require(failure.kind == .NONE);
        defer free_archive(*a);
        error := extract(*a, temp_dir(t));

        expect(error.kind == .PATH && error.entry == "link");
        free_archive(*error);
    }
} @test

duplicate_and_parent :: (t: *Test) {
    for second: string.["a", "./a", "a/child"] {
        items := Archive_Item.[.{ name = "a" }, .{ name = second }];
        bytes, error := encode_archive(items, .ZIP);

        require(error.kind == .NONE);
        defer free(bytes.data);
        a, failure := read_archive(bytes, .ZIP);

        require(failure.kind == .NONE);
        defer free_archive(*a);
        error = extract(*a, temp_dir(t));
        expect(error.kind == .DUPLICATE || error.kind == .PATH);
        free_archive(*error);
    }
} @test

seek_read :: (data: *void, offset: s64, buffer: [] u8) -> s64, string {
    source := cast(*[] u8) data;
    n := min(min(source.count - offset, buffer.count), 7);
    memcpy(buffer.data, source.data + offset, n);
    return n, "";
}

sink_write :: (data: *void, bytes: [] u8) -> s64, Io.Io_Error {
    total := cast(*s64) data;
    total.* += bytes.count;
    return bytes.count, .NONE;
}

seekable_and_borrowed :: (t: *Test) {
    data: [] u8 = .{ 70000, alloc(70000) };

    defer free(data.data);
    for data data[it_index] = cast,trunc(u8) it_index;

    for format: Archive_Format.[.ZIP, .TAR] {
        items := Archive_Item.[.{ name = "data", data = data }];
        bytes, err := encode_archive(items, format);

        require(err.kind == .NONE);
        defer free(bytes.data);
        a, error := read_archive(bytes, format);

        require(error.kind == .NONE);
        defer free_archive(*a);
        d, e := read_entry(*a, 0);

        require(e.kind == .NONE);
        expect(!d.owned && d.bytes.data == bytes.data + a.entries[0].offset);
        free_archive(*d);
        source := Archive_Source.{ data = *bytes, size = bytes.count, read_at = seek_read };
        b, failure := read_archive(source, format);

        require(failure.kind == .NONE);
        defer free_archive(*b);
        count: s64;
        copied := copy_entry(*b, 0, Io.Writer.{ data = *count, write_proc = sink_write });

        expect(copied.kind == .NONE && count == data.count);
        free_archive(*copied);
    }
} @test

crc_and_truncation :: (t: *Test) {
    bytes, err := encode_archive(Archive_Item.[.{ name = "x", data = cast([] u8) "hello" }], .ZIP);

    require(err.kind == .NONE);
    defer free(bytes.data);
    a, error := read_archive(bytes, .ZIP);

    require(error.kind == .NONE);
    bytes[a.entries[0].offset] ^= 1;
    d, failure := read_entry(*a, 0);

    expect(failure.kind == .CHECKSUM && failure.entry == "x");
    free_archive(*d);
    free_archive(*failure);
    free_archive(*a);

    for n: 0..bytes.count - 1 {
        part: [] u8 = .{ n, bytes.data };
        b, e := read_archive(part, .ZIP);

        expect(e.kind != .NONE);
        free_archive(*e);
        free_archive(*b);
    }
} @test

bombs_and_overlap :: (t: *Test) {
    payload: [] u8 = .{ 1048576, alloc(1048576) };

    defer free(payload.data);
    memset(payload.data, 0, payload.count);
    bytes, error := encode_archive(Archive_Item.[.{ name = "bomb", data = payload, method = .DEFLATE }], .ZIP);

    require(error.kind == .NONE);
    defer free(bytes.data);
    a, failure := read_archive(bytes, .ZIP, Archive_Limits.{ max_ratio = 10 });

    expect(failure.kind == .LIMIT && failure.entry == "bomb");
    free_archive(*failure);
    free_archive(*a);
    // A nested ZIP is data, never an instruction to recurse during extraction.
    current, err := encode_archive(Archive_Item.[.{ name = "payload", data = cast([] u8) "content" }], .ZIP);

    require(err.kind == .NONE);

    for 0..4 {
        next, e := encode_archive(Archive_Item.[.{ name = "nested.zip", data = current, method = .DEFLATE }], .ZIP);
        require(e.kind == .NONE);
        free(current.data);
        current = next;
    }

    defer free(current.data);
    nested, ne := read_archive(current, .ZIP);

    require(ne.kind == .NONE);
    defer free_archive(*nested);
    extracted := extract(*nested, temp_dir(t));

    expect(extracted.kind == .NONE);
    free_archive(*extracted);
    file, ok := read_entire_file(tprint("%/nested.zip", temp_dir(t)));

    expect(ok && file.count == nested.entries[0].size);
    free(file);
    // Every directory entry points at the same local record: declared totals alone cannot catch it.
    pair, pe := encode_archive(Archive_Item.[.{ name = "same", data = cast([] u8) "a" }, .{ name = "same", data = cast([] u8) "a" }], .ZIP);

    require(pe.kind == .NONE);
    defer free(pair.data);
    end := pair.count - 22;
    directory := cast(s64) pair[end + 16] + cast(s64) pair[end + 17] * 256;
    second := directory + 46 + 4;

    for 0..3 pair[second + 42 + it] = 0;

    overlapped, oe := read_archive(pair, .ZIP);

    expect(oe.kind == .INVALID && oe.entry == "same");
    free_archive(*oe);
    free_archive(*overlapped);
} @test

extract_regular :: (t: *Test) {
    bytes, error := encode_archive(Archive_Item.[.{ name = "a", kind = .DIRECTORY }, .{ name = "a/b", data = cast([] u8) "hello" }], .ZIP);

    require(error.kind == .NONE);
    defer free(bytes.data);
    a, e := read_archive(bytes, .ZIP);

    require(e.kind == .NONE);
    defer free_archive(*a);
    failure := extract(*a, temp_dir(t));

    require(failure.kind == .NONE);
    text, ok := read_entire_file(tprint("%/a/b", temp_dir(t)));

    expect(ok && text == "hello");
    free(text);
    again := extract(*a, temp_dir(t));

    expect(again.kind == .IO);
    free_archive(*again);
} @test

existing_symlink :: (t: *Test) {
    #if OS == .MACOS || OS == .LINUX {
        P :: #import "POSIX";
        outside := tprint("%/outside", temp_dir(t));
        dest := tprint("%/dest", temp_dir(t));

        require(make_directory_if_it_does_not_exist(outside));
        require(make_directory_if_it_does_not_exist(dest));
        require(P.symlink(temp_c_string(outside), temp_c_string(tprint("%/link", dest))) == 0);
        bytes, error := encode_archive(Archive_Item.[.{ name = "link/escape", data = cast([] u8) "bad" }], .TAR);

        require(error.kind == .NONE);
        defer free(bytes.data);
        a, e := read_archive(bytes, .TAR);

        require(e.kind == .NONE);
        defer free_archive(*a);
        failure := extract(*a, dest);

        expect(failure.kind == .IO);
        free_archive(*failure);
        _, ok := read_entire_file(tprint("%/escape", outside));

        expect(!ok);
    }
} @test

names_and_limits :: (t: *Test) {
    bytes, error := encode_archive(Archive_Item.[.{ name = "x" }], .ZIP);

    require(error.kind == .NONE);
    defer free(bytes.data);
    end := bytes.count - 22;
    central := cast(s64) bytes[end + 16] + cast(s64) bytes[end + 17] * 256;

    bytes[30] = 0x82;
    bytes[central + 46] = 0x82;
    invalid, e := read_archive(bytes, .ZIP);

    expect(e.kind == .INVALID);
    free_archive(*e);
    free_archive(*invalid);
    bytes[7] &= 0xf7;
    bytes[central + 9] &= 0xf7;
    cp437, ce := read_archive(bytes, .ZIP);

    require(ce.kind == .NONE);
    expect(cp437.entries[0].name == "é");
    free_archive(*cp437);

    for limits: Archive_Limits.[.{ max_entries = 0 }, .{ max_metadata = 0 }, .{ max_archive = 1 }, .{ max_entry = -1 }, .{ max_ratio = 0 }] {
        a, failure := read_archive(bytes, .ZIP, limits);
        expect(failure.kind == .LIMIT);
        free_archive(*failure);
        free_archive(*a);
    }
} @test

local_zip64_sizes :: (t: *Test) {
    // Local ZIP64 sizes must agree with the directory even when the directory uses 32-bit fields.
    h: [20] u8;
    h[0] = 1;
    h[2] = 16;
    h[4] = 1;
    h[12] = 1;
    bytes, error := encode_archive(Archive_Item.[.{ name = "x", data = cast([] u8) "z" }], .ZIP);

    require(error.kind == .NONE);
    defer free(bytes.data);
    count := bytes.count + 20;
    expanded: [] u8 = .{ count, alloc(count) };

    defer free(expanded.data);
    memcpy(expanded.data, bytes.data, 31);
    memcpy(expanded.data + 31, h.data, 20);
    memcpy(expanded.data + 51, bytes.data + 31, bytes.count - 31);
    // Inserted local extra shifts the central-directory offset; clear the descriptor flag.
    expanded[6] &= 0xf7;
    expanded[28] = 20;
    for 18..25 expanded[it] = 255;

    end := expanded.count - 22;
    old := cast(s64) expanded[end + 16];
    central := old + 20;

    expanded[end + 16] = cast(u8) central;
    expanded[central + 8] &= 0xf7;
    // Header CRC is copied from the directory; the now-unused descriptor is harmless padding.
    memcpy(expanded.data + 14, expanded.data + central + 16, 4);
    a, e := read_archive(expanded, .ZIP);

    require(e.kind == .NONE);
    free_archive(*a);
    expanded[35] = 2;
    bad, failure := read_archive(expanded, .ZIP);

    expect(failure.kind == .INVALID);
    free_archive(*failure);
    free_archive(*bad);
} @test

failed_source :: (data: *void, offset: s64, buffer: [] u8) -> s64, string {
    source := cast(*[] u8) data;
    // Metadata succeeds; entry reads later fail after the source is deliberately shortened.
    if offset >= source.count return 0, "closed";

    n := min(buffer.count, source.count - offset);

    memcpy(buffer.data, source.data + offset, n);

    return n, "";
}

source_failure_names_entry :: (t: *Test) {
    bytes, error := encode_archive(Archive_Item.[.{ name = "file", data = cast([] u8) "content" }], .ZIP);

    require(error.kind == .NONE);
    defer free(bytes.data);
    source_bytes := bytes;
    a, e := read_archive(Archive_Source.{ data = *source_bytes, size = bytes.count, read_at = failed_source }, .ZIP);

    require(e.kind == .NONE);
    defer free_archive(*a);
    source_bytes.count = 0;
    d, failure := read_entry(*a, 0);

    expect(failure.kind == .IO && failure.entry == "file");
    free_archive(*failure);
    free_archive(*d);
    count: s64;
    copy_failure := copy_entry(*a, 0, Io.Writer.{ data = *count, write_proc = sink_write });

    expect(copy_failure.kind == .IO && copy_failure.entry == "file");
    free_archive(*copy_failure);
} @test

write_failure_names_entry :: (t: *Test) {
    for format: Archive_Format.[.ZIP, .TAR] {
        bytes, error := encode_archive(Archive_Item.[.{ name = "file", data = cast([] u8) "content" }], format, Archive_Limits.{ max_archive = 12 });
        expect(bytes.count == 0 && error.kind == .LIMIT && error.entry == "file");
        free_archive(*error);
    }
} @test
