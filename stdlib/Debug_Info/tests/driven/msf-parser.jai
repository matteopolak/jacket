#import "Basic";
#import "File";
D :: #import "Debug_Info";

errors: s64;

capture :: (_text: string, _data: *void, info: Log_Info) {
    assert((info.common_flags & .ERROR) != 0);
    errors += 1;
}

main :: () {
    push_context {
        context.logger = capture;
        check_all();
    }

    assert(errors == 10);
    print("ok\n");
}

check_all :: () {
    for architecture: string.["x64", "arm64"] {
        bytes, read_ok := read_entire_file(tprint("%../../../../target/developer-tools-msf-%.pdb", #filepath, architecture));

        assert(read_ok);
        defer free(bytes);
        ok, msf := D.parse_msf(bytes);

        assert(ok && D.verify_integrity(msf));
        assert(cast(*u8) msf.header == bytes.data && msf.content.data == bytes.data);

        defer {
            for stream: msf.stream_table.stream_infos array_free(stream.stream_page_numbers);

            array_free(msf.stream_table.stream_infos);
            array_free(msf.siPnList.stream_page_numbers);
            array_free(msf.fpm.rgw);
            array_free(msf.fpm_freed.rgw);
        }

        for stream, index: msf.stream_table.stream_infos {
            assert(D.is_valid_user_stream_number(msf, cast(u32) index) == (index >= 5));
            actual, copy_ok := D.get_stream_data(msf, cast(u32) index);

            assert(copy_ok);
            defer free(actual.data);
            expected, expected_ok := read_entire_file(tprint("%../../../../target/developer-tools-msf-%-stream-%.bin", #filepath, architecture, index));

            assert(expected_ok && cast(string) actual == expected && stream.byte_count == expected.count);
            defer free(expected);
        }

        old := msf.stream_table.stream_infos[1].byte_count;

        msf.stream_table.stream_infos[1].byte_count += 1;
        assert(!D.verify_integrity(msf));
        msf.stream_table.stream_infos[1].byte_count = old;
        assert(D.verify_integrity(msf));

        for length: s64.[0, 30, 55, 4095, bytes.count - 1] {
            rejected_ok, rejected := D.parse_msf(.{ length, bytes.data });
            assert(!rejected_ok && rejected.header == null && rejected.stream_table.stream_infos.count == 0);
        }
    }

    fpm: D.Free_Page_Map;

    D.init(*fpm, 1, 4096);
    defer array_free(fpm.rgw);
    assert(fpm.iwMac == 1 && D.is_free_page(fpm, 0));
    D.alloc_page(*fpm, 0);
    D.ensure_room(*fpm, 129, false);
    assert(fpm.iwMac == 3 && !D.is_free_page(fpm, 128) && !D.is_free_page(fpm, 0));
    D.free_page(*fpm, 128);
    assert(D.is_free_page(fpm, 128));
    D.set_all(*fpm);
    assert(D.is_free_page(fpm, 0) && D.is_free_page(fpm, 191));
}
