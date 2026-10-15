#import "Basic";
#import "File";
D :: #import "Debug_Info";

#assert size_of(D.Codeview_Line_Info) == 8;
#assert offset_of(D.Codeview_Line_Info, "line_and_delta_and_flag") == 4;

main :: () {
    lines := u32.[0, 1, 0x654321, 0xfeefee, 0xffffff];
    for first_line: lines for delta : 0..127 for statement : 0..1 {
        row: D.Codeview_Line_Info;
        row.text_offset = 0x12345678;
        row.line_and_delta_and_flag = first_line | ((cast(u32) delta) << 24) | ((cast(u32) statement) << 31);
        assert(D.get_line_number(row) == first_line);
        assert(D.get_line_number_end_delta(row) == cast(u8) delta);
        assert(D.line_info_is_for_statement(row) == (statement != 0));
        assert(row.text_offset == 0x12345678);
    }

    for architecture: string.["x64", "arm64"] {
        bytes, ok := read_entire_file(tprint("%../../target/developer-tools-codeview-%.bin", #filepath, architecture));

        assert(ok && bytes.count == 8);
        defer free(bytes);
        row := (cast(*D.Codeview_Line_Info) bytes.data).*;

        assert(row.text_offset == 0x12345678);
        assert(D.get_line_number(row) == 0x654321);
        assert(D.get_line_number_end_delta(row) == 0x35);
        assert(D.line_info_is_for_statement(row));
    }

    print("ok\n");
}
