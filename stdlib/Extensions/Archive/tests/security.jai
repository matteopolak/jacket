#import "Basic";
#import "Extensions/Archive";
#import "Extensions/Testing";

seek_read :: (data: *void, offset: s64, buffer: [] u8) -> s64, string {
    source := cast(*[] u8) data;
    n := min(min(source.count - offset, buffer.count), 7);
    memcpy(buffer.data, source.data + offset, n);
    return n, "";
}

sink_write :: (data: *void, bytes: [] u8) -> bool {
    total := cast(*s64) data;
    total.* += bytes.count;
    return true;
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
        copied := copy_entry(*b, 0, Archive_Writer.{ data = *count, write_proc = sink_write });

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

put_le :: (out: *[..] u8, value: s64, width: s64) {
    for 0..width - 1 array_add(out, cast,trunc(u8) (value >> (8 * it)));
}

// One empty stored file "x" whose central record carries `records` ZIP64 extra records, each
// holding a single 8-byte size. The directory size field is 0xffffffff, so one record is needed.
zip_with_zip64_records :: (records: s64) -> [..] u8 {
    out: [..] u8;

    put_le(*out, 0x04034b50, 4);
    put_le(*out, 10, 2);
    put_le(*out, 0, 2);
    put_le(*out, 0, 2);
    put_le(*out, 0, 4);
    put_le(*out, 0, 4);
    put_le(*out, 0, 4);
    put_le(*out, 0, 4);
    put_le(*out, 1, 2);
    put_le(*out, 0, 2);
    array_add(*out, #char "x");

    central := out.count;

    put_le(*out, 0x02014b50, 4);
    put_le(*out, 45, 2);
    put_le(*out, 45, 2);
    put_le(*out, 0, 2);
    put_le(*out, 0, 2);
    put_le(*out, 0, 4);
    put_le(*out, 0, 4);
    put_le(*out, 0, 4);
    put_le(*out, 0xffffffff, 4);
    put_le(*out, 1, 2);
    put_le(*out, 12 * records, 2);
    put_le(*out, 0, 2);
    put_le(*out, 0, 2);
    put_le(*out, 0, 2);
    put_le(*out, 0, 4);
    put_le(*out, 0, 4);
    array_add(*out, #char "x");

    for 1..records {
        put_le(*out, 1, 2);
        put_le(*out, 8, 2);
        put_le(*out, 0, 8);
    }

    size := out.count - central;

    put_le(*out, 0x06054b50, 4);
    put_le(*out, 0, 2);
    put_le(*out, 0, 2);
    put_le(*out, 1, 2);
    put_le(*out, 1, 2);
    put_le(*out, size, 4);
    put_le(*out, central, 4);
    put_le(*out, 0, 2);

    return out;
}

duplicate_central_zip64 :: (t: *Test) {
    one := zip_with_zip64_records(1);

    defer array_free(one);
    a, ok := read_archive(one, .ZIP);

    require(ok.kind == .NONE);
    free_archive(*a);

    two := zip_with_zip64_records(2);

    defer array_free(two);
    bad, failure := read_archive(two, .ZIP);

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
    copy_failure := copy_entry(*a, 0, Archive_Writer.{ data = *count, write_proc = sink_write });

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
