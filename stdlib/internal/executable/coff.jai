Coff :: struct {
    header: *Coff_Header;
    optional_header: Coff_Optional_Header;
    section_headers: [] Coff_Section_Header;
    symbols: [] Coff_Symbol;
    string_table: [] u8;
    section_relocations: [][] Coff_Relocation;
    content: string;
}

Coff_Optional_Header :: struct {
    union {
        pe32: struct {
            base: *Coff_Optional_Header_Pe32;
            windows: *Coff_Optional_Header_Windows_Specific_Pe32;
        };
        pe32plus: struct {
            base: *Coff_Optional_Header_Pe32_Plus;
            windows: *Coff_Optional_Header_Windows_Specific_Pe32_Plus;
        };
    };
    data_directories: [] Coff_Data_Directory;
    debug_directory_entries: [] Coff_Debug_Directory_Entry;
}

is_coff :: (file_content: string) -> bool {
    if file_content.count < 20 return false;
    machine := coff_word(file_content.data, 2);
    // A COFF machine tag has no independent signature. Recognize supported tags.
    return machine == 0x8664 || machine == 0x14c || machine == 0xaa64;
}

parse_coff_file :: (path: string) -> success: bool, Coff {
    content, read_ok := F.read_entire_file(path);
    if !read_ok return false, .{};

    ok, value := parse_coff(content, path);
    if !ok B.free(content);

    return ok, value;
}

parse_coff :: (file_content: string, path := "(unspecified file)", verbose := false, start_offset := 0) -> success: bool, Coff {
    result: Coff;
    relocations: [..][] Coff_Relocation;
    transferred := false;

    defer if !transferred B.array_free(relocations);
    if start_offset < 0 || !coff_range(file_content.count, cast(u64) start_offset, 20) return false, .{};
    if cast(u64) (file_content.data + start_offset) % 4 != 0 return false, .{};

    header := cast(*Coff_Header) (file_content.data + start_offset);
    signature: string = .{ file_content.count - start_offset, cast(*u8) header };

    if !is_coff(signature) return false, .{};

    optional := cast(u64) start_offset + 20;
    section_offset := optional + header.optional_header_size;

    if !coff_range(file_content.count, optional, header.optional_header_size) return false, .{};
    if !coff_range(file_content.count, section_offset, (cast(u64) header.num_sections) * 40) return false, .{};
    if section_offset % 4 != 0 return false, .{};

    result.header = header;
    result.content = file_content;
    result.section_headers = .{ header.num_sections, cast(*Coff_Section_Header) (file_content.data + section_offset) };

    if header.optional_header_size != 0 {
        if header.optional_header_size < 2 return false, .{};

        magic := coff_word(file_content.data + optional, 2);
        directory_offset: u64;
        directory_count: u64;

        if magic == 0x20b {
            if header.optional_header_size < 112 || cast(u64) (file_content.data + optional) % 8 != 0 return false, .{};

            result.optional_header.pe32plus.base = cast(*Coff_Optional_Header_Pe32_Plus) (file_content.data + optional);
            result.optional_header.pe32plus.windows = cast(*Coff_Optional_Header_Windows_Specific_Pe32_Plus) (file_content.data + optional + 24);
            directory_count = result.optional_header.pe32plus.windows.number_of_rva_and_sizes;
            directory_offset = 112;
        } else if magic == 0x10b {
            if header.optional_header_size < 96 return false, .{};

            result.optional_header.pe32.base = cast(*Coff_Optional_Header_Pe32) (file_content.data + optional);
            result.optional_header.pe32.windows = cast(*Coff_Optional_Header_Windows_Specific_Pe32) (file_content.data + optional + 28);
            directory_count = result.optional_header.pe32.windows.number_of_rva_and_sizes;
            directory_offset = 96;
        } else return false, .{};

        if directory_count > (header.optional_header_size - directory_offset) / 8 return false, .{};

        result.optional_header.data_directories = .{ cast(s64) directory_count, cast(*Coff_Data_Directory) (file_content.data + optional + directory_offset) };
    }

    if header.symbol_table_offset != 0 {
        symbol_bytes := cast(u64) header.num_symbols;
        symbol_bytes *= 18;
        if !coff_range(file_content.count, header.symbol_table_offset, symbol_bytes + 4) return false, .{};

        string_offset := header.symbol_table_offset + symbol_bytes;
        string_size := coff_word(file_content.data + string_offset, 4);

        if string_size < 4 || !coff_range(file_content.count, string_offset, string_size) return false, .{};

        result.symbols = .{ header.num_symbols, cast(*Coff_Symbol) (file_content.data + header.symbol_table_offset) };
        result.string_table = .{ cast(s64) string_size, file_content.data + string_offset };
        index: s64;
        while index < result.symbols.count {
            symbol := *result.symbols[index];
            _, valid := get_symbol_name(result, index);
            if !valid || symbol.number_of_aux_symbols > result.symbols.count - index - 1 return false, .{};
            index += 1 + symbol.number_of_aux_symbols;
        }
    } else if header.num_symbols != 0 return false, .{};

    for section: result.section_headers {
        if !coff_range(file_content.count, section.pointer_to_raw_data, section.size_of_raw_data) return false, .{};
        if !coff_range(file_content.count, section.pointer_to_line_numbers, (cast(u64) section.number_of_line_numbers) * 6) return false, .{};
        if (section.characteristics & .LNK_NRELOC_OVFL) != 0 return false, .{};

        count := cast(u64) section.number_of_relocations;
        if !coff_range(file_content.count, section.pointer_to_relocations, count * 10) return false, .{};

        table: [] Coff_Relocation = .{ cast(s64) count, cast(*Coff_Relocation) (file_content.data + section.pointer_to_relocations) };
        for relocation: table {
            if relocation.symbol_index >= header.num_symbols return false, .{};
            if relocation.virtual_address > section.size_of_raw_data return false, .{};
        }

        _, valid := get_section_name(result, section);
        if !valid return false, .{};

        B.array_add(*relocations, table);
    }

    if result.optional_header.data_directories.count > 6 {
        debug := result.optional_header.data_directories[6];
        if debug.size != 0 {
            valid, offset := coff_rva(result, debug.virtual_address, debug.size);
            if !valid || debug.size % 28 != 0 || offset % 4 != 0 return false, .{};

            result.optional_header.debug_directory_entries = .{ debug.size / 28, cast(*Coff_Debug_Directory_Entry) (file_content.data + offset) };
            for entry: result.optional_header.debug_directory_entries {
                if !coff_range(file_content.count, entry.pointer_to_raw_data, entry.size_of_data) return false, .{};
            }
        }
    }

    result.section_relocations = .{ relocations.count, relocations.data };
    transferred = true;

    return true, result;
}

is_pe :: (file_content: string) -> bool, coff_start: int {
    if file_content.count < 64 || coff_word(file_content.data, 2) != MS_DOS_MAGIC_PREFIX return false, 0;

    offset := coff_word(file_content.data + 60, 4);
    if offset < 64 || !coff_range(file_content.count, offset, 24) return false, 0;
    if coff_word(file_content.data + offset, 4) != PE_MAGIC_PREFIX return false, 0;

    return true, cast(s64) offset + 4;
}

parse_pe_file :: (path: string) -> success: bool, Coff {
    content, read_ok := F.read_entire_file(path);
    if !read_ok return false, .{};

    ok, value := parse_pe(content, path);
    if !ok B.free(content);

    return ok, value;
}

parse_pe :: (file_content: string, path := "(unspecified file)", verbose := false) -> success: bool, Coff {
    valid, offset := is_pe(file_content);
    if !valid return false, .{};

    ok, value := parse_coff(file_content, path, verbose, offset);
    if !ok return false, .{};

    if value.header.optional_header_size == 0 {
        B.array_free(value.section_relocations);
        return false, .{};
    }

    return true, value;
}

get_pdata :: (using coff: Coff, section_index: int) -> [] Pdata_Entry {
    data := get_section_data(coff, section_index);
    if data.count % 12 != 0 return .{};

    entries: [..] Pdata_Entry;

    for 0..data.count / 12 - 1 {
        raw := data.data + it * 12;
        entry: Pdata_Entry;
        entry.begin_address = cast(u32) coff_word(raw, 4);
        entry.end_address = cast(u32) coff_word(raw + 4, 4);
        entry.unwind_info_address = cast(u32) coff_word(raw + 8, 4);
        if entry.begin_address >= entry.end_address {
            B.array_free(entries);
            return .{};
        }

        B.array_add(*entries, entry);
    }

    return .{ entries.count, entries.data };
}

get_xdata :: (using coff: Coff, section_index: int, offset: u64) -> success: bool, Unwind_Info {
    data := get_section_data(coff, section_index);
    if !coff_range(data.count, offset, 4) return false, .{};

    raw := data.data + offset;
    result: Unwind_Info;
    result.version = raw[0] & 7;
    flags := raw[0] >> 3;
    if result.version != 1 || flags > 4 || (flags & 4 != 0 && flags & 3 != 0) return false, .{};

    result.flags = cast(Unwind_Info_Flags) flags;
    result.prolog_size = raw[1];
    result.frame_register = raw[3] & 15;
    result.scaled_frame_register_offset = raw[3] >> 4;
    count := cast(u64) raw[2];
    if !coff_range(data.count, offset + 4, count * 2) return false, .{};

    cursor: u64;
    while cursor < count {
        code := raw + 4 + cursor * 2;
        operation := code[1] & 15;
        info := code[1] >> 4;
        slots: u64 = 1;

        if operation == 1 {
            if info > 1 return false, .{};
            slots = 2 + info;
        } else if operation == 4 || operation == 8 slots = 2;
        else if operation == 5 || operation == 9 slots = 3;
        else if operation == 6 || operation == 7 || operation > 10 return false, .{};

        if slots > count - cursor return false, .{};

        cursor += slots;
    }

    tail := offset + 4 + ((count + 1) / 2) * 4;

    if flags & 4 != 0 {
        if !coff_range(data.count, tail, 12) return false, .{};

        result.extra.chained = cast(*Pdata_Entry) B.alloc(size_of(Pdata_Entry));
        result.extra.chained.begin_address = cast(u32) coff_word(data.data + tail, 4);
        result.extra.chained.end_address = cast(u32) coff_word(data.data + tail + 4, 4);
        result.extra.chained.unwind_info_address = cast(u32) coff_word(data.data + tail + 8, 4);
    } else if flags & 3 != 0 {
        if !coff_range(data.count, tail, 4) return false, .{};
        result.extra.handler = cast(*Unwind_Info_Exception_Handler) B.alloc(size_of(Unwind_Info_Exception_Handler));
        result.extra.handler.address = cast(u32) coff_word(data.data + tail, 4);
    }

    codes: [..] u16;
    for 0..cast(s64) count - 1 B.array_add(*codes, cast(u16) coff_word(raw + 4 + it * 2, 2));

    result.unwind_codes = .{ codes.count, codes.data };

    return true, result;
}

get_section_data :: (coff: Coff, section_index: int) -> [] u8 {
    if section_index < 0 || section_index >= coff.section_headers.count return .{};
    section := coff.section_headers[section_index];
    if !coff_range(coff.content.count, section.pointer_to_raw_data, section.size_of_raw_data) return .{};
    return .{ section.size_of_raw_data, coff.content.data + section.pointer_to_raw_data };
}

get_symbol_name :: (using coff: Coff, symbol_index: int) -> string, bool {
    if symbol_index < 0 || symbol_index >= symbols.count return "", false;
    symbol := *symbols[symbol_index];
    if symbol.name.long_name.zeroes == 0 return get_string_table_entry(coff, symbol.name.long_name.offset);
    return string_from_zero_padded_array(symbol.name.short_name), true;
}

get_string_table_entry :: (using coff: Coff, offset: int) -> string, bool {
    if offset < 4 || offset >= string_table.count return "", false;

    for offset..string_table.count - 1 {
        if string_table[it] == 0 return .{ it - offset, string_table.data + offset }, true;
    }

    return "", false;
}

get_section_name :: (coff: Coff, header: Coff_Section_Header) -> string, bool {
    if header.name[0] != #char "/" return B.copy_temporary_string(string_from_zero_padded_array(header.name)), true;

    offset: s64;

    for 1..7 {
        c := header.name[it];
        if c == 0 break;
        if c < #char "0" || c > #char "9" return "", false;
        offset = offset * 10 + c - #char "0";
    }

    return get_string_table_entry(coff, offset);
}

#scope_file
B :: #import "Basic";
F :: #import "File";

coff_rva :: (coff: Coff, rva: u64, bytes: u64) -> bool, u64 {
    for section: coff.section_headers {
        if rva < section.virtual_address continue;

        delta := rva - section.virtual_address;
        if delta > section.size_of_raw_data || bytes > section.size_of_raw_data - delta continue;

        offset := section.pointer_to_raw_data + delta;
        if coff_range(coff.content.count, offset, bytes) return true, offset;
    }

    return false, 0;
}

coff_range :: (size: s64, offset: u64, bytes: u64) -> bool {
    if size < 0 || offset > cast(u64) size return false;
    return bytes <= cast(u64) size - offset;
}

coff_word :: (data: *u8, bytes: s64) -> u64 {
    result: u64;

    for 0..bytes - 1 {
        byte := cast(u64) data[it];
        result |= byte << (8 * it);
    }

    return result;
}
