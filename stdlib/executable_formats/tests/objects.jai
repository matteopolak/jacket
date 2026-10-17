#import "Basic";
E :: #import "executable_formats";
S :: #import "String";

Tracking :: struct {
    base: Allocator;
    live: s64;
}

track :: (mode: Allocator_Mode, size: s64, old_size: s64, old: *void, data: *void) -> *void {
    state := cast(*Tracking) data;
    result := state.base.proc(mode, size, old_size, old, state.base.data);

    if mode == .ALLOCATE && result state.live += 1;
    if mode == .RESIZE && !old && result state.live += 1;
    if mode == .FREE && old state.live -= 1;

    return result;
}

release :: (value: E.Mach_O) {
    array_free(value.load_commands);
    array_free(value.sections);
    array_free(value.symbols);
}

main :: () {
    state := Tracking.{ base = context.allocator };
    scoped := context;
    scoped.allocator = .{ track, *state };
    push_context scoped run_cases();
    assert(state.live == 0, "object parser leaked % allocations", state.live);
    print("ok\n");
}

run_cases :: () {
    assert(size_of(E.mach_header) == 28);
    assert(size_of(E.mach_header_64) == 32);
    assert(size_of(E.nlist) == 12 && size_of(E.nlist_64) == 16);
    raw_symbol: E.nlist_64;
    raw_symbol.n_un.n_strx = 37;
    assert(raw_symbol.n_un.n_strx == 37);
    assert(cast(u8) E.Elf_Endianness.MSB == 1);
    assert(size_of(E.Mach_Header) == 32);
    assert(size_of(E.Section_64) == 80);
    assert(size_of(E.Coff_Header) == 20);
    assert(size_of(E.Coff_Symbol) == 18);
    assert(size_of(E.Coff_Relocation) == 10);
    assert(size_of(E.Coff_Optional_Header) == 48);
    assert(size_of(E.Coff_Optional_Header_Windows_Specific_Pe32) == 68);
    assert(size_of(E.Coff_Optional_Header_Windows_Specific_Pe32_Plus) == 88);
    assert(size_of(E.Elf_Header_64) == 64);
    assert(size_of(E.Elf_Header_32) == 52);
    assert(size_of(E.Elf_Section_Header) == 64);
    assert(size_of(E.Elf_Program_Header_64) == 56);
    assert(size_of(E.Elf_Program_Header_32) == 32);
    assert(E.MH_MAGIC_64 == 0xfeedfacf);
    assert(cast(u32) E.Load_Command_Type.SEGMENT_64 == 0x19);
    assert(!E.is_macho("short"));
    ok, bad := E.parse_macho("short");

    assert(!ok && bad.content.count == 0 && bad.header == null);
    root := sprint("%/../../../target/executable-fixtures/", S.path_strip_filename(#file));

    defer free(root);
    path := sprint("%macho.o", root);

    defer free(path);
    parsed, macho := E.parse_macho_file(path);

    assert(parsed && macho.header.magic == E.MH_MAGIC_64);
    assert(macho.header.ncmds == 4 && macho.load_commands.count == 4);
    assert(macho.sections.count >= 5 && macho.symbols.count >= 3);
    assert(macho.debug_info_section && macho.debug_abbrev_section && macho.debug_line_section);
    found := false;

    for symbol: macho.symbols {
        if symbol.name == "_object_answer" {
            assert(symbol.type.type == .SECTION && symbol.section == 1);
            assert((symbol.flags & .EXTERNAL) != 0);
            assert(symbol.name.data >= macho.content.data && symbol.name.data < macho.content.data + macho.content.count);
            found = true;
        }
    }

    assert(found);

    for section: macho.sections {
        assert(cast(*u8) section >= macho.content.data && cast(*u8) section < macho.content.data + macho.content.count);
        bytes := E.get_section_data(macho, section);
        if E.get_section_name(section.*) == "__text" assert(bytes.count > 0 && bytes.data == macho.content.data + section.offset);
        if E.get_section_name(section.*) == "__bss" assert(bytes.count == 0);
    }

    for 0..macho.content.count - 1 {
        truncated: string = .{ it, macho.content.data };
        valid, partial := E.parse_macho(truncated);
        assert(!valid && partial.header == null && partial.sections.count == 0, "truncated prefix % accepted", it);
    }

    section: E.Section_64;

    assert(E.get_section_data(macho, *section).count == 0);
    release(macho);
    free(macho.content);

    for label: string.["commands-overrun", "zero-command", "sections-overrun", "symbol-name-overrun", "symbols-overflow", "relocations-overflow"] {
        broken := sprint("%%.o", root, label);
        defer free(broken);
        valid, empty := E.parse_macho_file(broken);
        assert(!valid && empty.content.count == 0 && empty.sections.count == 0 && empty.symbols.count == 0);
    }

    coff_path := sprint("%coff.o", root);

    defer free(coff_path);
    coff_ok, coff := E.parse_coff_file(coff_path);

    assert(coff_ok && coff.header.machine == .AMD64 && coff.section_headers.count > 2);
    coff_found := false;
    index: s64;

    while index < coff.symbols.count {
        name, name_ok := E.get_symbol_name(coff, index);
        assert(name_ok);
        if name == "object_answer" coff_found = true;
        index += 1 + coff.symbols[index].number_of_aux_symbols;
    }

    assert(coff_found);

    for section: coff.section_headers {
        name, valid := E.get_section_name(coff, section);

        assert(valid);

        if name == ".xdata" {
            unwind_ok, unwind := E.get_xdata(coff, it_index, 0);
            assert(unwind_ok && unwind.version == 1 && unwind.unwind_codes.count > 0);
            array_free(unwind.unwind_codes);
            if unwind.extra.handler free(unwind.extra.handler);
        }

        if name == ".pdata" {
            pdata := E.get_pdata(coff, it_index);
            assert(pdata.count > 0);
            array_free(pdata);
        }
    }

    for 0..coff.content.count - 1 {
        valid, partial := E.parse_coff(.{ it, coff.content.data });
        assert(!valid && partial.content.count == 0, "COFF prefix % accepted", it);
    }

    array_free(coff.section_relocations);
    free(coff.content);
    elf_path := sprint("%elf.o", root);

    defer free(elf_path);
    elf_ok, elf := E.parse_elf_file(elf_path);

    assert(elf_ok && elf.header.machine == .AMD_x64 && elf.sections.count > 3);
    assert(elf.text.count > 0 && elf.debug_info_section && elf.debug_line_section);
    elf_found := false;

    for symbol: elf.symbols {
        kind, name, valid := E.get_symbol_type_and_name(elf, symbol, elf.symbol_names);

        assert(valid);

        if name == "object_answer" {
            assert(kind == .FUNC && E.get_binding(symbol) == .GLOBAL);
            elf_found = true;
        }
    }

    assert(elf_found);

    for 0..elf.content.count - 1 {
        prefix: string = .{ it, elf.content.data };
        valid, partial := E.parse_elf(prefix);
        assert(!valid && partial.content.count == 0, "ELF prefix % accepted", it);
    }

    free(elf.content);
    elf_executable_path := sprint("%elf-executable", root);

    defer free(elf_executable_path);
    executable_ok, elf_executable := E.parse_elf_file(elf_executable_path);

    assert(executable_ok && elf_executable.program_header_table.count > 0 && elf_executable.header.type == .EXEC);
    for program: elf_executable.program_header_table assert(program.filesz <= program.memsz);

    free(elf_executable.content);
    fat_path := sprint("%universal.o", root);

    defer free(fat_path);
    fat_ok, fat := E.parse_apple_universal_file(fat_path);

    assert(fat_ok && fat.files.count == 2);

    for arch: fat.files {
        assert(E.is_macho(.{ arch.data.count, arch.data.data }));
        assert(arch.data.data >= fat.content.data && arch.data.data < fat.content.data + fat.content.count);
        child_ok, child := E.parse_macho(.{ arch.data.count, arch.data.data });

        assert(child_ok && child.symbols.count > 0);
        release(child);
    }

    array_free(fat.files);
    free(fat.content);

    for label: string.["fat-0-0", "fat-0-1", "fat-1-0", "fat-1-1"] {
        variant_path := sprint("%%.o", root, label);

        defer free(variant_path);
        valid, variant := E.parse_apple_universal_file(variant_path);

        assert(valid && variant.files.count == 1 && variant.files[0].cpu_type == .ARM);
        assert(variant.files[0].data.count == 3 && variant.files[0].data[0] == #char "a");

        for 0..variant.content.count - 1 {
            prefix_ok, partial := E.parse_apple_universal(.{ it, variant.content.data }, "prefix");
            assert(!prefix_ok && partial.files.count == 0);
        }

        array_free(variant.files);
        free(variant.content);
    }

    for label: string.["fat-reserved-0", "fat-reserved-1", "fat-overlap"] {
        malformed_path := sprint("%%.o", root, label);
        defer free(malformed_path);
        valid, empty := E.parse_apple_universal_file(malformed_path);
        assert(!valid && empty.content.count == 0 && empty.files.count == 0);
    }

    pe_path := sprint("%answer.exe", root);

    defer free(pe_path);
    pe_ok, pe := E.parse_pe_file(pe_path);

    assert(pe_ok && pe.optional_header.pe32plus.base.magic == .PE32_PLUS);
    assert(pe.optional_header.pe32plus.windows.image_base != 0 && pe.section_headers.count > 0);
    assert(pe.optional_header.debug_directory_entries.count > 0);

    for 0..pe.content.count - 1 {
        valid, partial := E.parse_pe(.{ it, pe.content.data });
        assert(!valid && partial.content.count == 0, "PE prefix % accepted", it);
    }

    array_free(pe.section_relocations);
    free(pe.content);
    elf_bad_path := sprint("%elf-overflow.o", root);

    defer free(elf_bad_path);
    elf_bad_ok, elf_bad := E.parse_elf_file(elf_bad_path);

    assert(!elf_bad_ok && elf_bad.content.count == 0);
    coff_bad_path := sprint("%coff-overflow.o", root);

    defer free(coff_bad_path);
    coff_bad_ok, coff_bad := E.parse_coff_file(coff_bad_path);

    assert(!coff_bad_ok && coff_bad.content.count == 0);
    coff_sections_bad_path := sprint("%coff-sections-overflow.o", root);

    defer free(coff_sections_bad_path);
    coff_sections_bad_ok, coff_sections_bad := E.parse_coff_file(coff_sections_bad_path);

    assert(!coff_sections_bad_ok && coff_sections_bad.content.count == 0);

    program_wire := cast(*u8) alloc(128);

    defer free(program_wire);
    for 0..127 program_wire[it] = 0;

    program_header := cast(*E.Elf_Header_64) program_wire;
    program_header.ident.magic = 0x464c457f;
    program_header.ident.class = ._64_BIT;
    program_header.ident.data = .LSB;
    program_header.ident.version = 1;
    program_header.version = 1;
    program_header.ehsize = 64;
    program_header.phnum = 1;
    program_header.phentsize = 56;
    program_header.program_header_offset = 64;
    program := cast(*E.Elf_Program_Header_64) (program_wire + 64);
    program.type = .NULL;
    program.offset = 0xffffffffffffffff;
    program.filesz = 0xffffffffffffffff;
    program.align = 3;
    null_ok, null_header, null_table := E.parse_elf_header_and_program_header_table(.{ 128, program_wire });

    assert(null_ok && null_header == program_header && null_table.count == 1);
    program.type = .NOTE;
    program.offset = 120;
    program.filesz = 1;
    program.align = 0;
    note_ok := E.parse_elf_header_and_program_header_table(.{ 128, program_wire });

    assert(note_ok);

    unwind_bytes: [16] u8;
    unwind_sections: [1] E.Coff_Section_Header;
    unwind_sections[0].size_of_raw_data = 16;
    unwind_coff: E.Coff;
    unwind_coff.section_headers = unwind_sections;
    unwind_coff.content = .{ 16, unwind_bytes.data };
    unwind_bytes[0] = 9; // Version 1, exception-handler tail at offset 4.
    unwind_bytes[4] = 0x34;
    unwind_bytes[5] = 0x12;
    handler_ok, handler := E.get_xdata(unwind_coff, 0, 0);

    assert(handler_ok && handler.extra.handler.address == 0x1234 && handler.unwind_codes.count == 0);
    free(handler.extra.handler);
    array_free(handler.unwind_codes);
    for unwind_bytes unwind_bytes[it_index] = 0;

    unwind_bytes[0] = 33; // Version 1, chained runtime-function tail.
    unwind_bytes[4] = 1;
    unwind_bytes[8] = 2;
    chained_ok, chained := E.get_xdata(unwind_coff, 0, 0);

    assert(chained_ok && chained.extra.chained.begin_address == 1 && chained.extra.chained.end_address == 2);
    free(chained.extra.chained);
    array_free(chained.unwind_codes);
    for unwind_bytes unwind_bytes[it_index] = 0;

    unwind_bytes[0] = 1;
    unwind_bytes[2] = 1;
    unwind_bytes[5] = 0x11; // ALLOC_LARGE requires three slots, only one is present.
    invalid_unwind_ok, invalid_unwind := E.get_xdata(unwind_coff, 0, 0);

    assert(!invalid_unwind_ok && invalid_unwind.unwind_codes.count == 0 && invalid_unwind.extra.handler == null);
}
