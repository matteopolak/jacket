// Driven by differential.py; jacket's test runner does not call this main.
#import "Basic";
#import "File";
#import "Extensions/Archive";

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 2);
    root := args[1];

    for format: Archive_Format.[.ZIP, .TAR, .TAR_GZIP] {
        suffix := ifx format == .ZIP then "zip" else ifx format == .TAR then "tar" else "tar.gz";

        for variant: 0..2 {
            if format != .ZIP && variant > 0 continue;

            text, ok := read_entire_file(tprint("%/python-%.%", root, variant, suffix));

            assert(ok);
            defer free(text);
            a, error := read_archive(cast([] u8) text, format);

            assert(error.kind == .NONE, tprint("%: %", suffix, error.message));
            defer free_archive(*a);
            assert(a.entries.count == 3);
            assert(a.entries[0].name == "dir/");
            assert(a.entries[1].name == "dir/世界.txt");
            assert(a.entries[2].name.count == 180);

            for a.entries {
                d, e := read_entry(*a, it_index);

                assert(e.kind == .NONE, tprint("%: %", suffix, e.message));
                if it_index == 1 assert(cast(string) d.bytes == "hello world\n");
                if it_index == 2 assert(cast(string) d.bytes == "long path\n");

                free_archive(*d);
            }

            if format != .ZIP {
                assert(a.entries[1].mtime == -123.5);
                assert(a.entries[2].mtime == 1234.0);
            }
        }

        long_bytes: [180] u8;
        for long_bytes long_bytes[it_index] = #char "a";

        long_name := cast(string) long_bytes;
        items := Archive_Item.[.{ name = "dir/", kind = .DIRECTORY }, .{ name = "dir/世界.txt", data = cast([] u8) "hello world\n", method = .DEFLATE }, .{ name = long_name, data = cast([] u8) "long path\n" }];
        output, error := encode_archive(items, format);

        assert(error.kind == .NONE);
        assert(write_entire_file(tprint("%/ours.%", root, suffix), cast(string) output));
        free(output.data);
    }

    print("interop ok\n");
}
