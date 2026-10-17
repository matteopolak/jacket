#import "Basic";
#import "File";
#load "../../../../target/developer-tools-msf-wire.jai";

main :: () {
    for architecture: string.["x64", "arm64"] {
        bytes, read_ok := read_entire_file(tprint("%../../../../target/developer-tools-msf-%.pdb", #filepath, architecture));

        assert(read_ok);
        defer free(bytes);
        view, error := tools_msf_decode(bytes);
        if error != .NONE print("MSF decode error: %\n", error);

        assert(error == .NONE);
        defer tools_msf_release(*view);
        assert(view.block_size == 4096 && view.streams.count >= 5);

        for stream, index: view.streams {
            expected, expected_ok := read_entire_file(tprint("%../../../../target/developer-tools-msf-%-stream-%.bin", #filepath, architecture, index));

            assert(expected_ok);
            defer free(expected);
            data, copy_error := tools_msf_copy_stream(view, index);

            assert(copy_error == .NONE);
            defer free(data.data);
            assert(cast(string) data == expected);
            assert(data.count == stream.size);
        }

        for length: 0..55 {
            bad, bad_error := tools_msf_decode(.{ length, bytes.data });
            assert(bad_error == .HEADER && bad.streams.count == 0);
        }

        bad, bad_error := tools_msf_decode(.{ bytes.count - 1, bytes.data });

        assert(bad_error == .HEADER && bad.streams.count == 0);
        original := bytes.data[0];

        bytes.data[0] = 0;
        bad, bad_error = tools_msf_decode(bytes);
        assert(bad_error == .HEADER && bad.streams.count == 0);
        bytes.data[0] = original;
        byte_offset := cast(s64) view.active_fpm * view.block_size;

        original = bytes.data[byte_offset];
        bytes.data[byte_offset] |= 1;
        bad, bad_error = tools_msf_decode(bytes);
        assert(bad_error == .FREE_PAGE && bad.streams.count == 0);
        bytes.data[byte_offset] = original;
        data, copy_error := tools_msf_copy_stream(view, -1);

        assert(copy_error == .STREAM && data.count == 0);
        data, copy_error = tools_msf_copy_stream(view, view.streams.count);
        assert(copy_error == .STREAM && data.count == 0);
    }

    synthetic, synthetic_ok := read_entire_file(tprint("%../../../../target/developer-tools-msf-synthetic.bin", #filepath));

    assert(synthetic_ok);
    defer free(synthetic);
    view, error := tools_msf_decode(synthetic);

    assert(error == .NONE && view.streams.count == 1024);
    defer tools_msf_release(*view);
    data, copy_error := tools_msf_copy_stream(view, 0);

    assert(copy_error == .NONE && data.count == 5000);
    defer free(data.data);
    for byte, index: data assert(byte == cast(u8) ((index * 13 + 7) % 256));

    empty, empty_error := tools_msf_copy_stream(view, 1023);

    assert(empty_error == .NONE && empty.count == 0);
    tools_msf_release(*view);

    for invalid_page: u32.[0, 1, 2, 3, 4, 7, 10, 12, 0xffffffff] {
        store_u32(synthetic, 7 * 4096 + 8, invalid_page);
        bad, bad_error := tools_msf_decode(synthetic);
        assert(bad_error == .PAGE && bad.streams.count == 0);
    }

    store_u32(synthetic, 7 * 4096 + 8, 8);
    store_u32(synthetic, 4 * 4096 + 4, 0xffffffff);
    bad, bad_error := tools_msf_decode(synthetic);

    // A nil stream consumes no page numbers; leaving its old page-list bytes
    // in the directory is a malformed trailing directory, not an unsupported
    // nil-stream encoding.
    assert(bad_error == .DIRECTORY && bad.streams.count == 0);
    store_u32(synthetic, 44, 4100);
    nil_view, nil_error := tools_msf_decode(synthetic);

    assert(nil_error == .NONE && nil_view.streams[0].size == -1);
    nil_data, nil_copy_error := tools_msf_copy_stream(nil_view, 0);

    assert(nil_copy_error == .STREAM && nil_data.count == 0);
    tools_msf_release(*nil_view);
    store_u32(synthetic, 44, 4108);
    store_u32(synthetic, 4 * 4096 + 4, 5000);
    store_u32(synthetic, 3 * 4096 + 4, 4);
    bad, bad_error = tools_msf_decode(synthetic);
    assert(bad_error == .PAGE && bad.streams.count == 0);
    store_u32(synthetic, 3 * 4096 + 4, 7);
    store_u32(synthetic, 4 * 4096, 0xffffffff);
    bad, bad_error = tools_msf_decode(synthetic);
    assert(bad_error == .DIRECTORY && bad.streams.count == 0);
    store_u32(synthetic, 4 * 4096, 0);
    store_u32(synthetic, 44, 4);
    no_streams, no_streams_error := tools_msf_decode(synthetic);

    assert(no_streams_error == .NONE && no_streams.streams.count == 0);
    tools_msf_release(*no_streams);
    print("ok\n");
}

store_u32 :: (content: string, offset: s64, value: u32) {
    for byte: 0..3 content.data[offset + byte] = cast(u8) ((value >> (byte * 8)) & 0xff);
}
