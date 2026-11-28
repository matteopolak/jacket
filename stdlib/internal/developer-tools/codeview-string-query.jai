// Returns the NUL-terminated string at offset `id` in `info.string_table`, and whether it was found.
//
// Fails for an offset past the table or a string with no terminator. The result points into the table; it is not a copy.
get_string_from_table :: (info: Codeview_Debug_Info, id: Codeview_String_Table_Id) -> string, success: bool {
    if info.string_table.count <= 0 || !info.string_table.data || id >= info.string_table.count return "", false;

    at := cast(s64) id;
    while at < info.string_table.count {
        if info.string_table[at] == 0 return .{ at - cast(s64) id, info.string_table.data + id }, true;
        at += 1;
    }

    return "", false;
}

// Returns the file name for the checksum entry `id`, and whether it was found.
//
// Fails if no entry has that ID, if two do (an error is logged), or if its name is not in the string table. The result points into the string table.
get_filename_for_file_id :: (info: Codeview_Debug_Info, id: Codeview_File_Checksum_Id) -> string, success: bool {
    if info.file_checksums.count < 0 || (info.file_checksums.count > 0 && !info.file_checksums.data) return "", false;

    found := false;
    filename_id: Codeview_String_Table_Id;

    for checksum: info.file_checksums {
        if checksum.id != id continue;

        if found {
            log_error("Debug_Info: ambiguous CodeView file checksum ID %\n", id);
            return "", false;
        }

        found = true;
        filename_id = checksum.filename_id;
    }

    if !found return "", false;

    return get_string_from_table(info, filename_id);
}

#scope_file
#import "Basic";
