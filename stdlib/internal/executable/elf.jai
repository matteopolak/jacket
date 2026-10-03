is_elf :: (file_content: string) -> bool {
    return file_content.count >= 4 && file_content[0] == 0x7f && file_content[1] == #char "E" && file_content[2] == #char "L" && file_content[3] == #char "F";
}

parse_elf_file :: (path: string) -> success: bool, elf: Elf {
    content, read_ok := F.read_entire_file(path);
    if !read_ok return false, .{};

    ok, value := parse_elf(content, path);
    if !ok B.free(content);

    return ok, value;
}

parse_elf_header_and_program_header_table :: (data: string, path := "(unspecified file)") -> bool, *Elf_Header_64, [] Elf_Program_Header_64 {
    if !is_elf(data) || data.count < 64 return false, null, .{};
    if data[4] != 2 || data[5] != 1 || data[6] != 1 || cast(u64) data.data % 8 != 0 return false, null, .{};

    header := cast(*Elf_Header_64) data.data;
    if header.version != 1 || header.ehsize != 64 return false, null, .{};

    count := cast(u64) header.phnum;
    if count == 0xffff {
        if header.section_header_entry_size != 64 || header.section_header_offset % 8 != 0 || !elf_range(data.count, header.section_header_offset, 64) return false, null, .{};
        first := cast(*Elf_Section_Header) (data.data + header.section_header_offset);
        count = first.info;
    }

    table: [] Elf_Program_Header_64;

    if count != 0 {
        if header.phentsize != 56 || header.program_header_offset % 8 != 0 || count > cast(u64) data.count / 56 return false, null, .{};
        if !elf_range(data.count, header.program_header_offset, count * 56) return false, null, .{};

        table = .{ cast(s64) count, cast(*Elf_Program_Header_64) (data.data + header.program_header_offset) };
        for entry: table {
            if entry.type == .NULL continue;
            if !elf_range(data.count, entry.offset, entry.filesz) return false, null, .{};
            if (entry.type == .LOAD || entry.type == .TLS) && entry.filesz > entry.memsz return false, null, .{};
            if !elf_alignment(entry.align) return false, null, .{};
            if entry.align > 1 && entry.vaddr % entry.align != entry.offset % entry.align return false, null, .{};
        }
    }

    return true, header, table;
}

parse_elf :: (file_content: string, path := "(unspecified file)") -> success: bool, elf: Elf {
    header_ok, header, programs := parse_elf_header_and_program_header_table(file_content, path);
    if !header_ok return false, .{};

    result: Elf;
    result.header = header;
    result.program_header_table = programs;
    result.content = file_content;
    result.debug_info_section_index = -1;
    result.debug_line_section_index = -1;

    if header.section_header_offset == 0 {
        if header.num_section_headers != 0 || header.section_names_section_index != 0 return false, .{};
        return true, result;
    }

    if header.section_header_entry_size != 64 || header.section_header_offset % 8 != 0 || !elf_range(file_content.count, header.section_header_offset, 64) return false, .{};

    first := cast(*Elf_Section_Header) (file_content.data + header.section_header_offset);
    count := cast(u64) header.num_section_headers;

    if count == 0 count = first.size;
    if count == 0 || count > cast(u64) file_content.count / 64 || !elf_range(file_content.count, header.section_header_offset, count * 64) return false, .{};

    result.sections = .{ cast(s64) count, first };
    name_index := cast(u64) header.section_names_section_index;
    if name_index == 0xffff name_index = first.link;
    if name_index >= count return false, .{};

    if name_index != 0 {
        names := *result.sections[name_index];
        if names.type != .STRTAB || !validate_section_header(names, file_content.count) return false, .{};
        result.section_names = .{ cast(s64) names.size, file_content.data + names.offset };
    }

    if first.type != .NULL return false, .{};

    for index: 1..result.sections.count - 1 {
        section := *result.sections[index];
        if !validate_section_header(section, file_content.count) return false, .{};
        if section.type == .REL return false, .{};

        if section.type == .SYMTAB || section.type == .DYNSYM {
            if section.entsize != 24 || section.size % 24 != 0 || section.offset % 8 != 0 || section.link >= count return false, .{};

            strings := *result.sections[section.link];
            if strings.type != .STRTAB || !validate_section_header(strings, file_content.count) return false, .{};

            symbols: [] Elf_Symbol = .{ cast(s64) (section.size / 24), cast(*Elf_Symbol) (file_content.data + section.offset) };
            table: string = .{ cast(s64) strings.size, file_content.data + strings.offset };

            if section.info > symbols.count return false, .{};

            for symbol: symbols {
                valid := get_string_from_table(symbol.name_offset, table);
                if !valid || symbol.section_index == 0xffff return false, .{};
                if symbol.section_index < 0xff00 && symbol.section_index >= count return false, .{};
            }

            if section.type == .SYMTAB {
                if result.symbols.data return false, .{};
                result.symbols = symbols;
                result.symbol_names = table;
            } else {
                if result.dynamic_symbols.data return false, .{};
                result.dynamic_symbols = symbols;
                result.dynamic_symbol_names = table;
            }
        }

        if section.type == .RELA {
            if section.entsize != 24 || section.size % 24 != 0 || section.offset % 8 != 0 || section.link >= count || section.info >= count return false, .{};

            linked := result.sections[section.link];
            if linked.type != .SYMTAB && linked.type != .DYNSYM return false, .{};
            if linked.entsize != 24 return false, .{};

            relocs: [] Elf_Relocation = .{ cast(s64) (section.size / 24), cast(*Elf_Relocation) (file_content.data + section.offset) };
            for relocation: relocs if cast(u32) relocation.symbol_id >= linked.size / 24 return false, .{};
        }

        if name_index != 0 {
            name, name_ok := get_section_name(result, section.*);
            if !name_ok return false, .{};

            if name == ".text" {
                result.text_section = section;
                result.text = get_section_data(result, section);
            }

            if name == ".debug_info" {
                result.debug_info_section = section;
                result.debug_info_section_index = index;
            }

            if name == ".debug_line" {
                result.debug_line_section = section;
                result.debug_line_section_index = index;
            }

            if name == ".debug_abbrev" result.debug_abbrev_section = section;
            if name == ".debug_str" result.debug_str_section = section;
            if name == ".debug_loc" result.debug_loc_section = section;
            if name == ".debug_ranges" result.debug_ranges_section = section;
        }
    }

    return true, result;
}

validate_section_header :: (section_header: *Elf_Section_Header, file_size: s64) -> success: bool {
    if !section_header || file_size < 0 return false;
    if !elf_alignment(section_header.addralign) return false;
    if section_header.type == .NOBITS return section_header.offset <= cast(u64) file_size;
    return elf_range(file_size, section_header.offset, section_header.size);
}

get_section_data :: (elf: Elf, section: *Elf_Section_Header) -> [] u8 {
    if !section || section.type == .NOBITS || !validate_section_header(section, elf.content.count) return .{};
    return .{ cast(s64) section.size, elf.content.data + section.offset };
}

get_relocations :: (elf: Elf, section_index: int) -> [] Elf_Relocation, bool {
    if section_index < 0 || section_index >= elf.sections.count return .{}, false;

    for section: elf.sections {
        if section.type != .RELA || section.info != section_index continue;
        if section.entsize != 24 || section.size % 24 != 0 || section.offset % 8 != 0 || !validate_section_header(*section, elf.content.count) return .{}, false;
        return .{ cast(s64) (section.size / 24), cast(*Elf_Relocation) (elf.content.data + section.offset) }, true;
    }

    return .{}, true;
}

get_symbol :: (elf: Elf, symbol_id: Elf_Symbol_Id) -> *Elf_Symbol {
    index := cast(u32) symbol_id;
    if index >= elf.symbols.count return null;
    return *elf.symbols[index];
}

get_type :: (symbol: Elf_Symbol) -> Elf_Symbol_Type {
    return cast(Elf_Symbol_Type) (symbol.info & 15);
}

get_binding :: (symbol: Elf_Symbol) -> Elf_Symbol_Binding {
    return cast(Elf_Symbol_Binding) (symbol.info >> 4);
}

get_visibility :: (symbol: Elf_Symbol) -> Elf_Symbol_Visibility {
    return cast(Elf_Symbol_Visibility) (symbol.other & 3);
}

get_symbol_type_and_name :: (elf: Elf, symbol: Elf_Symbol, symbol_names: string) -> Elf_Symbol_Type, name: string, success: bool {
    ok, name := get_string_from_table(symbol.name_offset, symbol_names);
    return get_type(symbol), name, ok;
}

get_section_name :: (elf: Elf, section: Elf_Section_Header) -> string, success: bool {
    ok, name := get_string_from_table(section.name_offset, elf.section_names);
    return name, ok;
}

get_string_from_table :: (index: int, table: string) -> success: bool, string {
    if index < 0 || index >= table.count return false, "";
    for index..table.count - 1 if table[it] == 0 return true, .{ it - index, table.data + index };
    return false, "";
}

#scope_file
B :: #import "Basic";
F :: #import "File";

elf_range :: (size: s64, offset: u64, bytes: u64) -> bool {
    if size < 0 || offset > cast(u64) size return false;
    return bytes <= cast(u64) size - offset;
}

elf_alignment :: (value: u64) -> bool {
    return value == 0 || (value & (value - 1)) == 0;
}
