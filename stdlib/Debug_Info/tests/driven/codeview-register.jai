#import "Basic";
#import "File";
D :: #import "Debug_Info";

main :: () {
    symbol: D.Codeview_Symbol_Defrange_Register_Rel;

    assert(symbol.gaps.count == 0 && symbol.range.range_bytes == 0);
    expected_offsets := u16.[0, 0, 0, 0, 1, 0xabc, 0xfff, 0xfff];
    expected_spilled := bool.[false, true, false, true, false, true, false, true];

    for architecture: string.["x64", "arm64"] {
        bytes, ok := read_entire_file(tprint("%../../../../target/developer-tools-codeview-register-%.bin", #filepath, architecture));

        assert(ok && bytes.count == expected_offsets.count * 2);
        defer free(bytes);

        for offset, i: expected_offsets {
            symbol.spilled_udt_member_and_offset_parent = (cast(*u16) (bytes.data + i * 2)).*;
            assert(D.get_spilled_udt_member(*symbol) == expected_spilled[i]);
            assert(D.get_offset_parent(*symbol) == offset);
        }
    }

    print("ok\n");
}
