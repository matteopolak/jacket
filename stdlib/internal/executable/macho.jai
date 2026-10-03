is_macho :: (file_content: string) -> bool {
    if file_content.count < 4 return false;
    magic := read_le(file_content.data, 4);
    return magic == MH_MAGIC || magic == MH_MAGIC_64 || magic == MH_CIGAM || magic == MH_CIGAM_64;
}

parse_macho_file :: (path: string) -> success: bool, Mach_O {
    content, read_ok := F.read_entire_file(path);
    if !read_ok return false, .{};

    ok, value := parse_macho(content, path);
    if !ok B.free(content);

    return ok, value;
}

parse_macho :: (file_content: string, path := "(unspecified file)") -> success: bool, Mach_O {
    result: Mach_O;
    commands: [..] *Load_Command;
    transferred := false;

    defer if !transferred {
        B.array_free(commands);
        B.array_free(result.sections);
        B.array_free(result.symbols);
    }

    // Public pointers describe native little-endian 64-bit records.
    if file_content.count < 32 || !aligned(file_content.data, 8) return false, .{};
    if read_le(file_content.data, 4) != MH_MAGIC_64 return false, .{};

    header := cast(*Mach_Header) file_content.data;
    if !range_ok(file_content.count, 32, header.sizeofcmds) return false, .{};
    if header.ncmds > header.sizeofcmds / 8 return false, .{};

    command_end := 32 + cast(s64) header.sizeofcmds;
    cursor: s64 = 32;
    symtab: *Symtab_Command;

    for 0..cast(s64) header.ncmds - 1 {
        if !range_ok(command_end, cast(u64) cursor, 8) return false, .{};

        command := cast(*Load_Command) (file_content.data + cursor);
        if command.cmdsize < 8 || command.cmdsize % 8 != 0 || !range_ok(command_end, cast(u64) cursor, command.cmdsize) return false, .{};

        if command.cmd == .SEGMENT_64 {
            if command.cmdsize < 72 return false, .{};

            segment := cast(*Segment_Command_64) command;
            if segment.nsects > (command.cmdsize - 72) / 80 return false, .{};
            if command.cmdsize != 72 + (cast(u64) segment.nsects) * 80 return false, .{};
            if !range_ok(file_content.count, segment.fileoff, segment.filesize) return false, .{};

            for index: 0..cast(s64) segment.nsects - 1 {
                section := cast(*Section_64) (file_content.data + cursor + 72 + index * 80);
                if !zero_fill(section.flags) && !range_ok(file_content.count, section.offset, section.size) return false, .{};
                if !range_ok(file_content.count, section.reloff, (cast(u64) section.nreloc) * 8) return false, .{};
                if section.align > 63 return false, .{};

                B.array_add(*result.sections, section);
            }
        } else if command.cmd == .SYMTAB {
            if command.cmdsize != 24 || symtab return false, .{};
            symtab = cast(*Symtab_Command) command;
            if !range_ok(file_content.count, symtab.symoff, (cast(u64) symtab.nsyms) * 16) return false, .{};
            if !range_ok(file_content.count, symtab.stroff, symtab.strsize) return false, .{};
        } else if command.cmd == .UUID {
            if command.cmdsize != 24 return false, .{};
        } else if command.cmd == .BUILD_VERSION {
            if command.cmdsize < 24 return false, .{};
            build := cast(*Build_Version_Command) command;
            if build.ntools > (command.cmdsize - 24) / 8 return false, .{};
        }

        B.array_add(*commands, command);
        cursor += command.cmdsize;
    }

    if cursor != command_end return false, .{};

    if symtab {
        strings: string = .{ symtab.strsize, file_content.data + symtab.stroff };

        for 0..cast(s64) symtab.nsyms - 1 {
            raw := file_content.data + symtab.symoff + it * 16;
            name_ok, name := terminated_string(strings, cast(s64) read_le(raw, 4));

            if !name_ok return false, .{};

            symbol: Mach_Symbol;
            symbol.name = name;
            symbol.section = raw[5];
            symbol.description = cast(u16) read_le(raw + 6, 2);
            symbol.value = read_le(raw + 8, 8);

            if raw[4] & N_STAB != 0 {
                symbol.type.symbolic_type = cast(Mach_Symbolic_Symbol_Type) raw[4];
                symbol.flags |= .SYMBOLIC;
            } else {
                symbol.type.type = cast(Mach_Symbol_Type) (raw[4] & N_TYPE);
                if symbol.type.type == .SECTION && (symbol.section == 0 || symbol.section > result.sections.count) return false, .{};
            }

            if raw[4] & N_EXT != 0 symbol.flags |= .EXTERNAL;
            if raw[4] & N_PEXT != 0 symbol.flags |= .PRIVATE_EXTERNAL;

            B.array_add(*result.symbols, symbol);
        }
    }

    for section: result.sections {
        name := string_from_zero_padded_array(section.sectname);

        if string_from_zero_padded_array(section.segname) != "__DWARF" continue;
        if name == "__debug_info" result.debug_info_section = section;
        if name == "__debug_abbrev" result.debug_abbrev_section = section;
        if name == "__debug_str" result.debug_str_section = section;
        if name == "__debug_line" result.debug_line_section = section;
    }

    result.header = header;
    result.content = file_content;
    result.load_commands = .{ commands.count, commands.data };
    transferred = true;

    return true, result;
}

get_section_data :: (macho: Mach_O, section: *Section_64) -> [] u8 {
    if !section return .{};

    found := false;
    for macho.sections if it == section found = true;
    if !found || zero_fill(section.flags) || !range_ok(macho.content.count, section.offset, section.size) return .{};

    return .{ cast(s64) section.size, macho.content.data + section.offset };
}

get_section_name :: (section: Section_64) -> string {
    return B.copy_temporary_string(string_from_zero_padded_array(section.sectname));
}

get_segment_name :: (section: Section_64) -> string {
    return B.copy_temporary_string(string_from_zero_padded_array(section.segname));
}

string_from_zero_padded_array :: (padded: [] u8) -> string {
    for padded if it == 0 return .{ it_index, padded.data };
    return .{ padded.count, padded.data };
}

#scope_file
B :: #import "Basic";
F :: #import "File";

read_le :: (data: *u8, bytes: s64) -> u64 {
    value: u64;

    for 0..bytes - 1 {
        byte := cast(u64) data[it];
        value |= byte << (8 * it);
    }

    return value;
}

aligned :: (data: *u8, alignment: u64) -> bool {
    return cast(u64) data % alignment == 0;
}

range_ok :: (size: s64, offset: u64, bytes: u64) -> bool {
    if size < 0 || offset > cast(u64) size return false;
    return bytes <= cast(u64) size - offset;
}

terminated_string :: (table: string, offset: s64) -> bool, string {
    if offset < 0 || offset >= table.count return false, "";
    for offset..table.count - 1 if table[it] == 0 return true, .{ it - offset, table.data + offset };
    return false, "";
}

zero_fill :: (flags: Section_Flags) -> bool {
    kind := cast(u32) flags & SECTION_TYPE;
    return kind == 1 || kind == 0xc || kind == 0x12;
}
