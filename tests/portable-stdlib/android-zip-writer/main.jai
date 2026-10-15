// Provenance: reviewed ZIP32 layouts, writer ownership/alignment/finalization.
using #import "Basic";
F :: #import "File";
A :: #import "Android/Toolchain";
S :: #import "String";

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    output := args[1];

    assert(size_of(A.Zip_Local_File_Header) == 30);
    assert(size_of(A.Zip_Central_Directory_Header) == 46);
    assert(size_of(A.Zip_End_Of_Central_Directory_Record) == 22);
    local: A.Zip_Local_File_Header;

    assert(cast(s64) *local.crc_32 - cast(s64) *local == 14);
    assert(cast(s64) *local.extra_field_length - cast(s64) *local == 28);
    central: A.Zip_Central_Directory_Header;

    assert(cast(s64) *central.external_file_attributes - cast(s64) *central == 38);
    assert(cast(s64) *central.relative_offset_of_local_header - cast(s64) *central == 42);
    assert(A.ZIP_VERSION_DEFAULT == 20);
    assert(A.ZIP_LOCAL_HEADER_SIZE == 30);
    w: A.Zip_Writer;

    A.zip_init_writer(*w, max_alignment = 4096);
    assert(w.write_crcs);
    assert(w.dumb_padding_memory.count == 4096);
    for byte: w.dumb_padding_memory assert(byte == 0);

    assert(A.zip_add_file_from_memory(*w, "stored data", "stored.txt"));
    assert(A.zip_add_file_from_memory(*w, "deflated data", "compressed.txt", .DEFLATE));
    assert(A.zip_add_file_from_memory(*w, "aligned data", "aligned.txt", alignment = 4096));
    assert(A.zip_add_file_from_memory(*w, "", "empty.txt", .DEFLATE));

    for length: .[65534, 65535, 65536, 131071] {
        bytes := alloc_string(length);
        for *byte, i: bytes byte.* = cast(u8) (i % 251);

        name := sprint("boundary-%", length);
        // Names must remain alive through finalization, as specified.
        temporary_name := copy_temporary_string(name);

        free(name);
        assert(A.zip_add_file_from_memory(*w, bytes, temporary_name, .DEFLATE));
        free(bytes);
    }

    assert(A.zip_add_file(*w, #file, "source.jai", .DEFLATE));
    for info: w.zip_file_infos assert(info.filename.data != null);

    A.zip_write_central_directory(*w);
    data := builder_to_string(*w.builder);
    path := S.join(output, "/archive.zip");

    defer free(path);
    assert(F.write_entire_file(path, data));
    free(data);
    implicit: A.Zip_Writer;

    assert(A.zip_add_file_from_memory(*implicit, "implicit init", "file"));
    A.zip_write_central_directory(*implicit);
    implicit_data := builder_to_string(*implicit.builder);
    implicit_path := S.join(output, "/implicit.zip");

    defer free(implicit_path);
    assert(F.write_entire_file(implicit_path, implicit_data));
    free(implicit_data);
    array_free(implicit.zip_file_infos);
    dummy: A.Zip_Writer;

    A.zip_init_writer(*dummy, do_crcs = false);
    assert(A.zip_add_file_from_memory(*dummy, "dummy", "dummy"));
    assert(dummy.zip_file_infos[0].crc == A.Zip_Writer.DUMMY_CRC);
    A.zip_write_central_directory(*dummy);
    dummy_data := builder_to_string(*dummy.builder);
    dummy_path := S.join(output, "/dummy.zip");

    defer free(dummy_path);
    assert(F.write_entire_file(dummy_path, dummy_data));
    free(dummy_data);
    print("ok\n");
}
