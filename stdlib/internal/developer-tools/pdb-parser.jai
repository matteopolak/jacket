parse_tpi_stream :: (msf: Msf, stream_index: UNSN, verbose := false) -> [] *Codeview_Type, success: bool {
    bytes, ok := get_stream_data(msf, stream_index);
    if !ok return .{}, false;

    defer free(bytes.data);
    if bytes.count < 56 return .{}, false;

    input := Line_Wire_Input.{ data = bytes, limit = bytes.count };
    version := line_wire_uint(*input, 4);
    header_size := line_wire_uint(*input, 4);
    begin := line_wire_uint(*input, 4);
    end := line_wire_uint(*input, 4);
    record_bytes := line_wire_uint(*input, 4);

    if version != 20040203 || header_size != 56 || begin != 0x1000 || end < begin || record_bytes > cast(u64) bytes.count - header_size return .{}, false;

    parsed_ok, records := parse_codeview_types(.{ cast(s64) record_bytes, bytes.data + header_size }, false, verbose);
    if !parsed_ok return .{}, false;

    if cast(u64) records.count != end - begin {
        cv_release_types(records);
        log_error("Debug_Info: TPI type-index range disagrees with record count\n");
        return .{}, false;
    }

    return records, true;
}

parse_pdb :: (file_content: string, path := "", verbose := false) -> success: bool, Pdb {
    result: Pdb;
    keep := false;

    defer if !keep {
        pdb_release(*result);
        log_error("Debug_Info: invalid or unsupported PDB %\n", path);
    }

    msf_ok, msf := parse_msf(file_content, path, verbose);
    if !msf_ok return false, .{};

    result.msf = msf;
    bytes, bytes_ok := get_stream_data(msf, 1);
    if !bytes_ok return false, .{};

    defer free(bytes.data);
    input := Line_Wire_Input.{ data = bytes, limit = bytes.count };

    result.header.version = cast(Pdb_Version) line_wire_uint(*input, 4);
    // Other public version constants do not establish the layout used here.
    if result.header.version != .VC70 return false, .{};

    result.header.signature = cast(u32) line_wire_uint(*input, 4);
    result.header.age = cast(u32) line_wire_uint(*input, 4);
    result.signature.Data1 = cast(u32) line_wire_uint(*input, 4);
    result.signature.Data2 = cast(u16) line_wire_uint(*input, 2);
    result.signature.Data3 = cast(u16) line_wire_uint(*input, 2);
    for *byte: result.signature.Data4 byte.* = cast(u8) line_wire_uint(*input, 1);

    string_size := line_wire_uint(*input, 4);
    if input.failed || string_size > cast(u64) (input.limit - input.cursor) return false, .{};

    if string_size {
        result.name_table.buffer = .{ cast(s64) string_size, cast(*u8) alloc(cast(s64) string_size) };
        if !result.name_table.buffer.data return false, .{};
        memcpy(result.name_table.buffer.data, bytes.data + input.cursor, cast(s64) string_size);
    }

    input.cursor += cast(s64) string_size;
    count := line_wire_uint(*input, 4);
    capacity := line_wire_uint(*input, 4);

    if input.failed || !capacity || count > capacity return false, .{};

    map := *result.name_table.string_to_name_map;
    if !pdb_read_bits(*input, capacity, *map.present) || !pdb_read_bits(*input, capacity, *map.deleted) return false, .{};
    if cardinality(map.present) != count || count > cast(u64) (input.limit - input.cursor) / 8 return false, .{};

    for word, index: map.present.words {
        if index < map.deleted.words.count && (word & map.deleted.words[index]) != 0 return false, .{};
    }

    for 0..cast(s64) count - 1 {
        string_offset := line_wire_uint(*input, 4);
        stream := cast(u32) line_wire_uint(*input, 4);

        if string_offset > 0x7fffffff || string_offset >= string_size || !stream_exists(msf, stream) return false, .{};

        name_input := Line_Wire_Input.{ data = result.name_table.buffer, cursor = cast(s64) string_offset, limit = cast(s64) string_size };
        name := line_wire_string(*name_input);

        if name_input.failed || !name.count return false, .{};

        for key: map.keys {
            previous := Line_Wire_Input.{ data = result.name_table.buffer, cursor = cast(s32) key, limit = cast(s64) string_size };
            if line_wire_string(*previous) == name return false, .{};
        }

        array_add(*map.keys, cast(type_of(map.keys[0])) string_offset);
        array_add(*map.values, cast(type_of(map.values[0])) stream);
    }

    if (input.limit - input.cursor) % 4 return false, .{};

    while input.cursor < input.limit && !input.failed {
        feature := line_wire_uint(*input, 4);
        if feature == {
            case 0; // Observed legacy separator in official SDK-generated PDBs.
            case 20091201, 20140508; result.contains_id_stream = true;
            case 0x4d544f4e; result.no_type_merge = true;
            case 0x494e494d; result.minimal_debug_info = true;
            case; return false, .{};
        }
    }

    if input.failed return false, .{};

    if !result.minimal_debug_info {
        tpi, tpi_ok := parse_tpi_stream(msf, 2, verbose);
        if !tpi_ok return false, .{};

        result.tpi = tpi;
        if result.contains_id_stream {
            ipi, ipi_ok := parse_tpi_stream(msf, 4, verbose);
            if !ipi_ok return false, .{};
            result.ipi = ipi;
        }
    }

    keep = true;

    return true, result;
}

#scope_file
#import "Basic";

UNSN :: u32;

pdb_read_bits :: (input: *Line_Wire_Input, capacity: u64, output: *Int_Set) -> bool {
    words := line_wire_uint(input, 4);
    if input.failed || words > (capacity + 31) / 32 || words > cast(u64) (input.limit - input.cursor) / 4 return false;

    if words {
        output.words = .{ cast(s64) words, cast(*u32) alloc(cast(s64) words * 4) };
        if !output.words.data return false;
        for *word: output.words word.* = cast(u32) line_wire_uint(input, 4);

        if words * 32 > capacity {
            last := output.words[output.words.count - 1];
            if (cast(u64) last >> (capacity % 32)) != 0 return false;
        }
    }

    return !input.failed;
}

pdb_release :: (pdb: *Pdb) {
    free(pdb.name_table.buffer.data);
    array_free(pdb.name_table.string_to_name_map.keys);
    array_free(pdb.name_table.string_to_name_map.values);
    free(pdb.name_table.string_to_name_map.present.words.data);
    free(pdb.name_table.string_to_name_map.deleted.words.data);
    cv_release_types(pdb.tpi);
    cv_release_types(pdb.ipi);
    msf_release(*pdb.msf);
    pdb.* = .{};
} @Releases
