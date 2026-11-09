#import "Basic";
#import "Extensions/Testing";
#import "Extensions/Archive";

roundtrip :: (t: *Test) {
    items := Archive_Item.[.{ name = "hello.txt", data = cast([] u8) "hello hello hello", method = .DEFLATE }];

    for format: Archive_Format.[.ZIP, .TAR, .TAR_GZIP] {
        bytes, error := encode_archive(items, format);

        assert(error.kind == .NONE);
        archive, failure := read_archive(bytes, format);

        assert(failure.kind == .NONE && archive.entries.count == 1, tprint("%: % % at %", format, failure.kind, failure.message, failure.offset));
        value, err := read_entry(*archive, 0);

        assert(err.kind == .NONE && cast(string) value.bytes == "hello hello hello");
        free_archive(*value);
        free_archive(*archive);
        free(bytes.data);
    }

    print("ok\n");
} @test

zip64_count :: (t: *Test) {
    items: [] Archive_Item = .{ 65535, alloc(65535 * size_of(Archive_Item)) };

    defer free(items.data);
    for *items it.* = .{ name = "x" };

    bytes, error := encode_archive(items, .ZIP);

    require(error.kind == .NONE);
    defer free(bytes.data);
    // The count sentinel requires both a ZIP64 end record and a locator.
    expect(bytes[bytes.count - 22 - 20] == 0x50 && bytes[bytes.count - 22 - 20 + 2] == 6);
    expect(bytes[bytes.count - 22 + 8] == 255 && bytes[bytes.count - 22 + 9] == 255);
    a, e := read_archive(bytes, .ZIP);

    require(e.kind == .NONE);
    expect(a.entries.count == items.count);
    free_archive(*a);
} @test @timeout(60)

collect_bytes :: (data: *void, bytes: [] u8) -> bool {
    out := cast(*[..] u8) data;
    for bytes array_add(out, it);
    return true;
}

// A caller's own sink works on every target, wasm included.
custom_sink :: (t: *Test) {
    items := Archive_Item.[.{ name = "a.txt", data = cast([] u8) "streamed", method = .DEFLATE }];

    for format: Archive_Format.[.ZIP, .TAR, .TAR_GZIP] {
        out: [..] u8;
        error := write_archive(Archive_Writer.{ data = *out, write_proc = collect_bytes }, items, format);

        require(error.kind == .NONE);
        archive, failure := read_archive(.{ out.count, out.data }, format);

        require(failure.kind == .NONE);
        copied: [..] u8;

        error = copy_entry(*archive, 0, Archive_Writer.{ data = *copied, write_proc = collect_bytes });

        expect(error.kind == .NONE && cast(string) (cast([] u8) copied) == "streamed");
        free_archive(*archive);
        array_free(copied);
        array_free(out);
    }
} @test
