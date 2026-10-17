#import "Basic";
#import "File";
D :: #import "Debug_Info";
#load "../../../../target/developer-tools-codeview-string-oracle.jai";

errors: s64;

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    assert(cast(u16) D.Codeview_Symbol_Type.RECTYPE_MAX == 0x1169);
    assert(cast(u16) D.Codeview_Symbol_Type.RECTYPE_LAST == 0x1168);
    assert(cast(u16) D.Codeview_Symbol_Type.RECTYPE_PAD == 0x1269);
    assert(cast(u16) D.Codeview_Leaf_Type.TYPE_LAST == 0x151e);
    assert(cast(u16) D.Codeview_Leaf_Type.TYPE_MAX == 0x151d);
    assert(cast(u16) D.Codeview_Leaf_Type.ID_LAST == 0x1608);
    assert(cast(u16) D.Codeview_Leaf_Type.ID_MAX == 0x1607);
    assert(D.Codeview_Leaf_Type.ENDOFLEAFRECORD == D.Codeview_Leaf_Type.VFTABLE);
    assert(cast(u16) D.Codeview_Symbol_Type.REGISTER_16t == 2);
    info: D.Codeview_Debug_Info;
    missing, missing_ok := D.get_string_from_table(info, 0);

    assert(!missing_ok && missing.count == 0);
    table := u8.[0, #char "a", 0, #char "b", #char "c", 0];

    info.string_table = .{ table.count, table.data };
    first, first_ok := D.get_string_from_table(info, 1);

    assert(first_ok && first == "a" && first.data == *table[1]);
    second, second_ok := D.get_string_from_table(info, 3);

    assert(second_ok && second == "bc" && second.data == *table[3]);
    missing, missing_ok = D.get_string_from_table(info, 0xffffffff);
    assert(!missing_ok && missing.count == 0);
    info.string_table.count -= 1;
    missing, missing_ok = D.get_string_from_table(info, 3);
    assert(!missing_ok && missing.count == 0);
    info.string_table.count += 1;
    array_add(*info.file_checksums, D.Codeview_File_Checksum.{ id = 24, filename_id = 3 });
    defer array_free(info.file_checksums);
    filename, filename_ok := D.get_filename_for_file_id(info, 24);

    assert(filename_ok && filename == "bc");
    filename, filename_ok = D.get_filename_for_file_id(info, 0);
    assert(!filename_ok && filename.count == 0);
    array_add(*info.file_checksums, info.file_checksums[0]);

    push_context {
        context.logger = capture;
        filename, filename_ok = D.get_filename_for_file_id(info, 24);
        assert(!filename_ok && filename.count == 0);
    }

    assert(errors == 1);
    check_native_codeview_strings();
    print("ok\n");
}
