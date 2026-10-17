#import "Basic";
D :: #import "Debug_Info";
E :: #import "executable_formats";
F :: #import "File";
S :: #import "String";

main :: () {
    #assert !#exists(E.Guid);
    #assert !#exists(E.Debug_Info);
    assert(size_of(D.Guid) == 16 && align_of(D.Guid) == 4);
    guid_info := cast(*Type_Info_Struct) type_info(D.Guid);

    assert(guid_info.members.count == 4);
    assert(guid_info.members[0].name == "data1" && guid_info.members[0].offset_in_bytes == 0);
    assert(guid_info.members[1].name == "data2" && guid_info.members[1].offset_in_bytes == 4);
    assert(guid_info.members[2].name == "data3" && guid_info.members[2].offset_in_bytes == 6);
    assert(guid_info.members[3].name == "data4" && guid_info.members[3].offset_in_bytes == 8);
    canonical: D.Guid = .{ 0x12345678, 0x9abc, 0xdef0, .[1, 2, 3, 4, 5, 6, 7, 8] };
    raw: E.Coff_Raw_Codeview_Info;
    raw.rsds = .[#char "R", #char "S", #char "D", #char "S"];
    raw.pdb_guid = canonical;
    raw.pdb_age = 3;
    #assert type_of(raw.pdb_guid) == D.Guid;
    assert(type_info(type_of(raw.pdb_guid)) == type_info(D.Guid));
    assert(size_of(E.Coff_Raw_Codeview_Info) == 24 && align_of(E.Coff_Raw_Codeview_Info) == 4);
    raw_info := cast(*Type_Info_Struct) type_info(E.Coff_Raw_Codeview_Info);

    assert(raw_info.name == "Coff_Raw_Codeview_Info" && raw_info.members.count == 3);
    assert(raw_info.members[0].name == "rsds" && raw_info.members[0].offset_in_bytes == 0);
    assert(raw_info.members[1].name == "pdb_guid" && raw_info.members[1].offset_in_bytes == 4);
    assert(raw_info.members[2].name == "pdb_age" && raw_info.members[2].offset_in_bytes == 20);
    assert(raw_info.members[1].type == type_info(D.Guid));
    bytes := cast(*u8) *raw;

    assert(bytes[0] == 82 && bytes[3] == 83 && bytes[4] == 0x78 && bytes[7] == 0x12);
    assert(bytes[8] == 0xbc && bytes[9] == 0x9a && bytes[10] == 0xf0 && bytes[11] == 0xde);
    assert(bytes[12] == 1 && bytes[19] == 8 && bytes[20] == 3);
    info: E.Coff_Codeview_Info;
    info.base = raw;
    info.pdb_path = "independent.pdb";
    assert(info.pdb_guid.data1 == canonical.data1 && info.pdb_age == 3 && info.pdb_path == "independent.pdb");
    assert(size_of(E.Coff_Codeview_Info) == 40 && align_of(E.Coff_Codeview_Info) == 8);
    derived_info := cast(*Type_Info_Struct) type_info(E.Coff_Codeview_Info);

    assert(derived_info.name == "Coff_Codeview_Info" && derived_info.members.count == 2);
    assert(derived_info.members[0].name == "base" && derived_info.members[0].offset_in_bytes == 0);
    assert(derived_info.members[1].name == "pdb_path" && derived_info.members[1].offset_in_bytes == 24);
    assert(derived_info.members[0].type == type_info(E.Coff_Raw_Codeview_Info));

    root := sprint("%/../../../target/executable-fixtures/", S.path_strip_filename(#file));

    defer free(root);
    path := sprint("%answer.exe", root);

    defer free(path);
    pe_ok, pe := E.parse_pe_file(path);

    assert(pe_ok && pe.optional_header.debug_directory_entries.count > 0);
    oracle_path := sprint("%codeview-oracle.bin", root);

    defer free(oracle_path);
    oracle, oracle_ok := F.read_entire_file(oracle_path);

    assert(oracle_ok && oracle.count >= 25);
    defer free(oracle);
    found := false;

    for entry: pe.optional_header.debug_directory_entries {
        if entry.type != .TYPE_CODEVIEW continue;

        assert(entry.size_of_data >= 25 && entry.pointer_to_raw_data <= pe.content.count - entry.size_of_data);
        data := pe.content.data + entry.pointer_to_raw_data;
        // Copy bytes into an aligned public ABI record; wire locations need not align.
        native: E.Coff_Raw_Codeview_Info;
        destination := cast(*u8) *native;

        for 0..23 destination[it] = data[it];
        for 0..23 assert(destination[it] == oracle[it], "CodeView header disagrees with llvm-readobj at byte %", it);

        assert(native.rsds[0] == #char "R" && native.rsds[1] == #char "S" && native.rsds[2] == #char "D" && native.rsds[3] == #char "S");
        assert(native.pdb_age == 1);
        decoded: E.Coff_Codeview_Info;
        decoded.base = native;
        encoded_path: [] u8 = .{ entry.size_of_data - 24, data + 24 };

        decoded.pdb_path = E.string_from_zero_padded_array(encoded_path);
        assert(decoded.pdb_path.count > 0 && decoded.pdb_path.count < encoded_path.count);
        assert(S.ends_with(decoded.pdb_path, "answer.pdb"));
        oracle_name: [] u8 = .{ oracle.count - 24, oracle.data + 24 };

        assert(decoded.pdb_path == E.string_from_zero_padded_array(oracle_name));
        as_canonical: D.Guid = decoded.pdb_guid;

        assert(as_canonical.data1 == native.pdb_guid.data1);
        found = true;
    }

    assert(found);
    array_free(pe.section_relocations);
    free(pe.content);
    print("ok\n");
}
