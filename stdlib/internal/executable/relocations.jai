apply_relocations :: (elf: Elf, section_index: int, data: [] u8) -> bool {
    if !elf.header || data.count < 0 || section_index < 0 || section_index >= elf.sections.count return false;

    target := elf.sections[section_index];
    if target.size > cast(u64) data.count return false;

    offsets: [..] u64;
    values: [..] u64;
    widths: [..] s64;

    defer B.array_free(offsets);
    defer B.array_free(values);
    defer B.array_free(widths);

    for table: elf.sections {
        if table.type != .RELA || table.info != section_index continue;
        if table.entsize != 24 || table.size % 24 != 0 || table.offset % 8 != 0 || !rel_range(elf.content.count, table.offset, table.size) return false;
        if table.link >= elf.sections.count return false;

        symbols := elf.sections[table.link];
        if (symbols.type != .SYMTAB && symbols.type != .DYNSYM) || symbols.entsize != 24 || symbols.size % 24 != 0 || symbols.offset % 8 != 0 return false;
        if !rel_range(elf.content.count, symbols.offset, symbols.size) return false;

        for 0..cast(s64) (table.size / 24) - 1 {
            relocation := cast(*Elf_Relocation) (elf.content.data + table.offset + it * 24);
            kind := cast(u32) relocation.type;

            if kind == 0 || (elf.header.machine == .ARM_v8 && kind == 256) continue;

            width: s64;
            relative := false;
            signed_result := false;
            mixed_result := false;

            if elf.header.machine == .AMD_x64 {
                if kind == 1 width = 8;
                else if kind == 2 || kind == 4 {
                    width = 4;
                    relative = true;
                    signed_result = true;
                } else if kind == 10 width = 4;
                else if kind == 11 {
                    width = 4;
                    signed_result = true;
                } else return false;
            } else if elf.header.machine == .ARM_v8 {
                if kind == 257 width = 8;
                else if kind == 258 {
                    width = 4;
                    mixed_result = true;
                } else if kind == 259 {
                    width = 2;
                    mixed_result = true;
                } else if kind == 260 {
                    width = 8;
                    relative = true;
                    signed_result = true;
                } else if kind == 261 || kind == 314 {
                    width = 4;
                    relative = true;
                    signed_result = true;
                } else if kind == 262 {
                    width = 2;
                    relative = true;
                    signed_result = true;
                } else return false;
            } else return false;

            offset := relocation.address;

            if elf.header.type != .REL {
                if offset < target.address return false;
                offset -= target.address;
            }

            if !rel_range(cast(s64) target.size, offset, cast(u64) width) return false;

            symbol_index := cast(u32) relocation.symbol_id;
            if symbol_index >= symbols.size / 24 return false;

            symbol := cast(*Elf_Symbol) (elf.content.data + symbols.offset + (cast(u64) symbol_index) * 24);
            address := symbol.value;

            if symbol.section_index == 0 || symbol.section_index == 0xfff2 return false;

            if symbol.section_index != 0xfff1 {
                if symbol.section_index >= elf.sections.count return false;

                if elf.header.type == .REL {
                    base := elf.sections[symbol.section_index].address;
                    if address > 0xffffffffffffffff - base return false;
                    address += base;
                }
            }

            if address > 0x7fffffffffffffff return false;

            valid, value := rel_add(cast(s64) address, relocation.addend);
            if !valid return false;

            if relative {
                if target.address > 0x7fffffffffffffff || offset > 0x7fffffffffffffff - target.address return false;
                place := cast(s64) (target.address + offset);
                valid, value = rel_add(value, -place);
                if !valid return false;
            }

            if mixed_result {
                if width == 4 && (value < -0x80000000 || value > 0xffffffff) return false;
                if width == 2 && (value < -0x8000 || value > 0xffff) return false;
            } else if !rel_fits(value, width, signed_result) return false;

            for previous: offsets if offset < previous + cast(u64) widths[it_index] && previous < offset + cast(u64) width return false;

            B.array_add(*offsets, offset);
            B.array_add(*values, cast(u64) value);
            B.array_add(*widths, width);
        }
    }

    for offsets rel_write(data.data + it, values[it_index], widths[it_index]);

    return true;
}

apply_relocations :: (coff: Coff, section_index: int, data: [] u8) {
    if !apply_coff(coff, section_index, data) B.print("COFF relocation rejected: invalid, unsupported or unresolved relocation\n");
}

#scope_file
B :: #import "Basic";

apply_coff :: (coff: Coff, section_index: s64, data: [] u8) -> bool {
    if !coff.header || data.count < 0 || coff.header.machine != .AMD64 || section_index < 0 || section_index >= coff.section_headers.count || section_index >= coff.section_relocations.count return false;

    target := coff.section_headers[section_index];
    if target.size_of_raw_data > data.count return false;

    offsets: [..] u64;
    values: [..] u64;
    widths: [..] s64;

    defer B.array_free(offsets);
    defer B.array_free(values);
    defer B.array_free(widths);

    for relocation: coff.section_relocations[section_index] {
        kind := cast(u16) relocation.type;
        if kind == 0 continue;
        if relocation.symbol_index >= coff.symbols.count return false;

        symbol := coff.symbols[relocation.symbol_index];
        if symbol.section_number == 0 return false;

        address := cast(u64) symbol.value;

        if symbol.section_number != 0xffff {
            if symbol.section_number > coff.section_headers.count return false;
            address += coff.section_headers[symbol.section_number - 1].virtual_address;
        }

        width: s64 = 4;
        if kind == 1 width = 8;
        if kind == 10 width = 2;
        if kind > 11 return false;

        offset := cast(u64) relocation.virtual_address;
        if !rel_range(target.size_of_raw_data, offset, cast(u64) width) return false;

        raw := rel_read(data.data + offset, width);
        if raw > 0x7fffffffffffffff return false;

        addend := cast(s64) raw;
        signed_result := kind >= 4 && kind <= 9;

        if signed_result && raw >= 0x80000000 addend -= 0x100000000;
        if kind == 10 address = symbol.section_number;
        if kind == 11 address = symbol.value;

        if kind == 1 || kind == 2 {
            if coff.optional_header.pe32plus.base {
                image_base: u64;
                if coff.optional_header.pe32plus.base.magic == .PE32_PLUS image_base = coff.optional_header.pe32plus.windows.image_base;
                else image_base = coff.optional_header.pe32.windows.image_base;
                if address > 0xffffffffffffffff - image_base return false;

                address += image_base;
            }
        }

        if address > 0x7fffffffffffffff return false;

        ok, value := rel_add(cast(s64) address, addend);
        if !ok return false;

        if signed_result {
            place := (cast(s64) target.virtual_address) + (cast(s64) offset) + 4 + kind - 4;
            ok, value = rel_add(value, -place);
            if !ok return false;
        }

        if !rel_fits(value, width, signed_result) return false;
        for previous: offsets if offset < previous + cast(u64) widths[it_index] && previous < offset + cast(u64) width return false;

        B.array_add(*offsets, offset);
        B.array_add(*values, cast(u64) value);
        B.array_add(*widths, width);
    }

    for offsets rel_write(data.data + it, values[it_index], widths[it_index]);

    return true;
}

rel_add :: (a: s64, b: s64) -> bool, s64 {
    if b > 0 && a > 0x7fffffffffffffff - b return false, 0;
    if b < 0 && a < (-0x7fffffffffffffff - 1) - b return false, 0;
    return true, a + b;
}

rel_fits :: (value: s64, width: s64, signed_result: bool) -> bool {
    if width == 8 return true;

    if width == 4 {
        if signed_result return value >= -0x80000000 && value <= 0x7fffffff;
        return value >= 0 && value <= 0xffffffff;
    }

    if signed_result return value >= -0x8000 && value <= 0x7fff;

    return value >= 0 && value <= 0xffff;
}

rel_range :: (size: s64, offset: u64, bytes: u64) -> bool {
    if size < 0 || offset > cast(u64) size return false;
    return bytes <= cast(u64) size - offset;
}

rel_read :: (data: *u8, width: s64) -> u64 {
    result: u64;

    for 0..width - 1 {
        byte := cast(u64) data[it];
        result |= byte << (8 * it);
    }

    return result;
}

rel_write :: (data: *u8, value: u64, width: s64) {
    for 0..width - 1 data[it] = cast(u8) ((value >> (8 * it)) & 0xff);
}
