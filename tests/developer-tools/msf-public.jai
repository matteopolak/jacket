// Reviewed Debug_Info MSF layout/read APIs; original native streams use SDK byte oracles.
#import "Basic";
#import "File";
D :: #import "Debug_Info";
#load "../../target/developer-tools-msf-wire.jai";

#run {
    record: D.Msf;

    assert(size_of(record.header.*) == 4096 && size_of(record.header.data) == 60);
    assert(type_of(record.header.data.valid_fpm_page_no) == u32);
    assert(type_of(record.header.data.page_count) == u32);
    assert(type_of(record.parms.cbitsFpm) == s32);
    assert(type_of(record.parms.pnMax) == u32);
    procedure := cast(*Type_Info_Procedure) type_info(type_of(D.get_stream_data));

    assert(procedure.argument_types[1] == type_info(u32));
    assert(D.is_msf == D.is_pdb);
}

errors: s64;

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    for architecture: string.["x64", "arm64"] {
        bytes, ok := read_entire_file(tprint("%../../target/developer-tools-msf-%.pdb", #filepath, architecture));

        assert(ok && D.is_msf(bytes) && D.is_pdb(bytes));
        defer free(bytes);
        view, decode_error := tools_msf_decode(bytes);

        assert(decode_error == .NONE);
        defer tools_msf_release(*view);
        msf: D.Msf;
        msf.content = bytes;
        msf.header = cast(type_of(msf.header)) bytes.data;
        msf.parms.page_size_in_bytes = 4096;
        msf.stream_table.page_size_in_bytes = 4096;
        assert(size_of(msf.header.*) == 4096);
        assert(size_of(msf.header.data) == 60);
        assert(cast(*u8) *msf.header.data.page_size - bytes.data == 32);
        assert(msf.header.data.page_size == 4096);
        array_resize(*msf.stream_table.stream_infos, view.streams.count + 1);

        defer {
            for stream: msf.stream_table.stream_infos array_free(stream.stream_page_numbers);
            array_free(msf.stream_table.stream_infos);
        }

        for stream, index: view.streams {
            info := *msf.stream_table.stream_infos[index];
            info.byte_count = cast(s32) stream.size;
            for page: stream.pages array_add(*info.stream_page_numbers, page);

            assert(D.is_valid_stream_number(msf, cast(u32) index));
            assert(D.stream_exists(msf, cast(u32) index));
            assert(D.get_stream_size(msf, cast(u32) index) == stream.size);
            actual, copy_ok := D.get_stream_data(msf, cast(u32) index);

            assert(copy_ok);
            defer free(actual.data);
            expected, expected_ok := read_entire_file(tprint("%../../target/developer-tools-msf-%-stream-%.bin", #filepath, architecture, index));

            assert(expected_ok && cast(string) actual == expected);
            defer free(expected);

            if actual.count > 0 {
                assert(actual.data != bytes.data + cast(s64) stream.pages[0] * 4096);
                saved := actual[0];
                actual[0] ^= 0xff;
                assert(bytes[cast(s64) stream.pages[0] * 4096] == saved);
            }
        }

        nil_index := cast(u32) view.streams.count;

        assert(msf.stream_table.stream_infos[nil_index].byte_count == -1);
        assert(D.is_valid_stream_number(msf, nil_index) && !D.stream_exists(msf, nil_index));
        assert(!D.is_valid_stream_number(msf, nil_index + 1));
        assert(!D.stream_exists(msf, 0xffffffff));
        assert(D.is_valid_page_number(msf, 0));
        assert(D.is_valid_page_number(msf, msf.header.data.page_count - 1));
        assert(!D.is_valid_page_number(msf, msf.header.data.page_count));
        assert(!D.is_valid_page_number(msf, 0xffffffff));

        push_context {
            context.logger = capture;
            data, data_ok := D.get_stream_data(msf, nil_index);

            assert(!data_ok && !data.count && !data.data);
            data, data_ok = D.get_stream_data(msf, 0xffffffff);
            assert(!data_ok && !data.count && !data.data);
            assert(D.get_stream_size(msf, nil_index) == -1);
            // Structurally malformed caller-owned stream descriptors cannot overread.
            first := *msf.stream_table.stream_infos[0];
            saved_size := first.byte_count;

            first.byte_count = 1;
            saved_pages := first.stream_page_numbers;

            first.stream_page_numbers = .{};
            data, data_ok = D.get_stream_data(msf, 0);
            assert(!data_ok && !data.count && !data.data);
            first.stream_page_numbers = saved_pages;
            first.byte_count = saved_size;

            // An invalid page number must fail before allocating/copying.
            for info: msf.stream_table.stream_infos {
                if info.stream_page_numbers.count == 0 continue;

                saved_page := info.stream_page_numbers[0];

                info.stream_page_numbers[0] = msf.header.data.page_count;
                data, data_ok = D.get_stream_data(msf, cast(u32) it_index);
                assert(!data_ok && !data.count && !data.data);
                info.stream_page_numbers[0] = saved_page;
                break;
            }

            saved_content := msf.content;

            msf.content.count -= 1;
            data, data_ok = D.get_stream_data(msf, 0);
            assert(!data_ok && !data.count && !data.data);
            msf.content = saved_content;
        }

        for length: 0..29 assert(!D.is_msf(.{ length, bytes.data }));

        magic := bytes.data[0];

        bytes.data[0] = 0;
        assert(!D.is_msf(bytes));
        bytes.data[0] = magic;
    }

    empty: D.Msf;

    assert(!D.is_valid_page_number(empty, 0));
    assert(!D.is_valid_stream_number(empty, 0));
    assert(errors == 10);
    print("ok\n");
}
