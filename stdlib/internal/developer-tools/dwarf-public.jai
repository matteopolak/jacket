parse_dwarf_debug_info :: (debug_info_data: [] u8, abbrev: [] u8, line_programs: [] u8, debug_string_data: [] u8, verbose := false) -> Debug_Info, success: bool {
    result: Debug_Info;
    keep := false;

    defer if !keep dwarf_release_info(*result);
    if !dwarf_bytes_valid(debug_info_data) || !dwarf_bytes_valid(abbrev) || !dwarf_bytes_valid(line_programs) || !dwarf_bytes_valid(debug_string_data) return .{}, false;

    parse: Parse_Context;
    parse.debug_string_data = debug_string_data;
    parse.verbose = verbose;
    offset: s64;
    while offset < debug_info_data.count {
        unit, ok := parse_compilation_unit_v4(*parse, result.compilation_units.count, offset, debug_info_data, abbrev, line_programs);
        if !ok {
            log_error("Debug_Info: invalid or unsupported DWARF4 compilation unit at %\n", offset);
            return .{}, false;
        }

        array_add(*result.compilation_units, unit);
        input := Line_Wire_Input.{ data = debug_info_data, cursor = offset, limit = debug_info_data.count };
        length := line_wire_uint(*input, 4);

        offset += 4 + cast(s64) length;
    }

    keep = true;

    return result, true;
}

parse_compilation_unit_v4 :: (using parse_context: *Parse_Context, cu_index: int, offset: int, full_cu_data: [] u8, abbrev: [] u8, line_programs: [] u8) -> Compilation_Unit, success: bool {
    result: Compilation_Unit;
    keep := false;

    defer if !keep dwarf_release_unit(*result);
    if !parse_context || cu_index < 0 || !dwarf_bytes_valid(full_cu_data) || !dwarf_bytes_valid(abbrev) || !dwarf_bytes_valid(line_programs) ||
        !dwarf_bytes_valid(parse_context.debug_string_data) || offset < 0 || offset > full_cu_data.count return .{}, false;
    if parse_context.current_compilation_unit || parse_context.depth || parse_context.subprogram_stack.count || parse_context.block_stack.count return .{}, false;

    parse_context.current_compilation_unit = *result;
    defer parse_context.current_compilation_unit = null;
    input := Line_Wire_Input.{ data = full_cu_data, cursor = offset, limit = full_cu_data.count };
    length := line_wire_uint(*input, 4);

    if input.failed || length >= 0xfffffff0 || length > cast(u64) (input.limit - input.cursor) return .{}, false;

    input.limit = input.cursor + cast(s64) length;
    version := line_wire_uint(*input, 2);
    abbrev_offset := line_wire_uint(*input, 4);
    address_size := cast(s64) line_wire_uint(*input, 1);

    if input.failed || version != 4 || (address_size != 4 && address_size != 8) return .{}, false;

    abbreviations: [..] Dwarf_Abbreviation;

    defer {
        for item: abbreviations array_free(item.attributes);
        array_free(abbreviations);
    }

    if !dwarf_read_abbreviations(abbrev, abbrev_offset, *abbreviations) return .{}, false;

    dies: [..] Dwarf_Die;
    parents: [..] s64;

    defer array_free(dies);
    defer array_free(parents);
    root_closed := false;

    while input.cursor < input.limit {
        die: Dwarf_Die;
        die.offset = input.cursor;
        code := line_wire_uleb(*input);

        if input.failed return .{}, false;

        if code == 0 {
            if parents.count == 0 return .{}, false;
            parents.count -= 1;
            if parents.count == 0 root_closed = true;
            continue;
        }

        if root_closed || (dies.count > 0 && parents.count == 0) return .{}, false;

        definition: *Dwarf_Abbreviation;
        for *item: abbreviations if item.code == code definition = item;
        if !definition return .{}, false;

        die.tag = definition.tag;
        die.parent = -1;
        if parents.count > 0 die.parent = parents[parents.count - 1];
        if dies.count == 0 && die.tag != .COMPILE_UNIT return .{}, false;

        for attribute: definition.attributes {
            value := dwarf_read_form(*input, attribute.form, address_size, offset, parse_context.debug_string_data);
            if input.failed || !dwarf_assign_attribute(*die, attribute.name, value) return .{}, false;
        }

        array_add(*dies, die);
        if definition.children array_add(*parents, dies.count - 1);
        else if dies.count == 1 root_closed = true;
    }

    if input.failed || !root_closed || parents.count || dies.count == 0 return .{}, false;
    // Resolve inherited descriptive attributes before projecting any public record.
    if !dwarf_resolve_dies(dies) return .{}, false;

    root := dies[0];

    result.producer = root.producer;
    result.language = cast(Dwarf_Language) root.language;
    result.name = root.name;
    result.statement_list = root.statement_list;
    result.low_pc = root.low_pc;
    result.high_pc = root.high_pc;

    if root.has_statement_list {
        table, ok := line_wire_decode(line_programs, cast(s64) root.statement_list, address_size);
        if !ok return .{}, false;
        defer line_wire_release(*table);
        if !dwarf_project_lines(table, *result.line_info) return .{}, false;
    }

    for *die: dies {
        if dwarf_is_type(die.tag) array_add(*result.types, Debug_Type.{ offset = die.offset, tag = die.tag, name = die.name, size = die.size, base_type = cast(Dwarf_Base_Type) die.encoding });

        if die.tag == .SUBPROGRAM {
            sub: Subprogram;
            sub.name = die.name;
            sub.linkage_name = die.linkage_name;
            sub.external = die.external;
            sub.decl_file_id = cast(Dwarf_File_Id) die.decl_file;
            sub.decl_line = die.decl_line;
            sub.type_id = die.type_id;
            sub.is_main = die.is_main;
            sub.calling_convention = die.calling_convention;
            sub.low_pc = die.low_pc;
            sub.high_pc = die.high_pc;
            block: Lexical_Block = .{ low_pc = sub.low_pc, high_pc = sub.high_pc };
            _, root_block := BA.bucket_array_add(*sub.all_blocks, block);

            sub.root_block = root_block;
            _, subprogram := BA.bucket_array_add(*result.subprograms, sub);

            die.subprogram = subprogram;
            die.subprogram.root_block = root_block;
            die.block = root_block;
        } else if die.parent >= 0 {
            parent := dies[die.parent];

            die.subprogram = parent.subprogram;
            die.block = parent.block;
            if die.tag == .LEXICAL_BLOCK && die.subprogram {
                block: Lexical_Block = .{ low_pc = die.low_pc, high_pc = die.high_pc };
                _, child_block := BA.bucket_array_add(*die.subprogram.all_blocks, block);
                die.block = child_block;
                array_add(*parent.block.children, die.block);
            }

            if die.tag == .VARIABLE || die.tag == .FORMAL_PARAMETER {
                declaration: Declaration;
                declaration.tag = die.tag;
                declaration.name = die.name;
                declaration.decl_file_id = cast(Dwarf_File_Id) die.decl_file;
                declaration.decl_line = die.decl_line;
                declaration.type_id = die.type_id;
                declaration.kind = .VARIABLE;
                if die.tag == .FORMAL_PARAMETER declaration.kind = .PARAMETER;
                if die.block array_add(*die.block.declarations, declaration);
                else if die.tag == .VARIABLE array_add(*result.globals, declaration);
            }
        }
    }

    keep = true;

    return result, true;
}

get_filename_for_file_id :: (info: Line_Info, file_id: Dwarf_File_Id) -> string {
    if !file_id || file_id > info.files.count || !info.files.data {
        log_error("Debug_Info: invalid DWARF file ID\n");
        return "";
    }

    return get_filename(info, info.files[file_id - 1]);
}

get_filename :: (info: Line_Info, file_info: File_Info) -> string {
    path := file_info.path;
    if path.count < 0 || (path.count && !path.data) return "";
    if !path.count || path[0] == #char "/" || !file_info.dir_id return path;

    if info.dirs.count < 0 || file_info.dir_id > cast(u64) info.dirs.count || !info.dirs.data {
        log_error("Debug_Info: invalid DWARF directory ID\n");
        return "";
    }

    directory := info.dirs[file_info.dir_id - 1];
    if directory.count < 0 || (directory.count && !directory.data) {
        log_error("Debug_Info: invalid DWARF directory string\n");
        return "";
    }

    if !directory.count return path;
    if directory[directory.count - 1] == #char "/" return sprint("%%", directory, path);

    return sprint("%/%", directory, path);
}

#scope_file
#import "Basic";
BA :: #import "Bucket_Array";

Parse_Context :: struct {
    debug_info: Debug_Info;
    debug_string_data: [] u8;
    depth: int;
    verbose: bool;
    current_compilation_unit: *Compilation_Unit;
    subprogram_stack: [..] *Subprogram;
    block_stack: [..] *Lexical_Block;
}

Dwarf_Abbreviation :: struct {
    code: u64;
    tag: Dwarf_Tag;
    children: bool;
    attributes: [..] Dwarf_Attribute;
}

Dwarf_Attribute :: struct {
    name: u64;
    form: u64;
}

Dwarf_Value :: struct {
    number: u64;
    text: string;
    address: bool;
    reference: bool;
    text_value: bool;
    block_value: bool;
    negative: bool;
}

Dwarf_Die :: struct {
    offset: s64;
    parent: s64;
    tag: Dwarf_Tag;
    name: string;
    linkage_name: string;
    producer: string;
    language: u64;
    statement_list: u64;
    has_statement_list: bool;
    low_pc: u64;
    high_pc: u64;
    high_relative: bool;
    size: u64;
    encoding: u64;
    decl_file: u32;
    decl_line: u32;
    type_id: u32;
    external: bool;
    is_main: bool;
    calling_convention: u8;
    origin: u64;
    specification: u64;
    present: u64;
    resolved: bool;
    origin_die: *Dwarf_Die;
    specification_die: *Dwarf_Die;
    subprogram: *Subprogram;
    block: *Lexical_Block;
}

dwarf_bytes_valid :: (data: [] u8) -> bool {
    return data.count >= 0 && (data.count == 0 || data.data != null);
}

dwarf_read_abbreviations :: (data: [] u8, offset: u64, output: *[..] Dwarf_Abbreviation) -> bool {
    if offset >= cast(u64) data.count return false;

    input := Line_Wire_Input.{ data = data, cursor = cast(s64) offset, limit = data.count };
    while !input.failed {
        code := line_wire_uleb(*input);

        if input.failed return false;
        if !code return true;
        for item: output.* if item.code == code return false;

        item: Dwarf_Abbreviation;
        item.code = code;
        item.tag = cast(Dwarf_Tag) line_wire_uleb(*input);
        children := line_wire_uint(*input, 1);
        if input.failed || children > 1 return false;

        item.children = children == 1;
        array_add(output, item);
        current := *output.*[output.count - 1];

        while !input.failed {
            name := line_wire_uleb(*input);
            form := line_wire_uleb(*input);

            if input.failed || (!name && form) || (name && !form) return false;
            if !name break;
            for attr: current.attributes if attr.name == name return false;

            array_add(*current.attributes, Dwarf_Attribute.{ name, form });
        }
    }

    return false;
}

dwarf_read_form :: (input: *Line_Wire_Input, encoded: u64, width: s64, cu_offset: s64, strings: [] u8) -> Dwarf_Value {
    result: Dwarf_Value;
    form := encoded;

    for 0..15 {
        if form != 0x16 break;
        form = line_wire_uleb(input);
    }

    count: s64;
    relative := false;
    block := false;

    if form == {
        case 0x01;
            count = width;
            result.address = true;
        case 0x03;
            count = cast(s64) line_wire_uint(input, 2);
            block = true;
        case 0x04;
            count = cast(s64) line_wire_uint(input, 4);
            block = true;
        case 0x05; count = 2;
        case 0x06, 0x17; count = 4;
        case 0x07, 0x20; count = 8;
        case 0x08;
            result.text_value = true;
            result.text = line_wire_string(input);
            return result;
        case 0x09, 0x18;
            count = cast(s64) line_wire_uleb(input);
            block = true;
        case 0x0a;
            count = cast(s64) line_wire_uint(input, 1);
            block = true;
        case 0x0b, 0x0c; count = 1;
        case 0x0d;
            number := line_wire_sleb(input);

            result.number = cast(u64) number;
            result.negative = number < 0;

            return result;
        case 0x0e;
            result.text_value = true;
            position := line_wire_uint(input, 4);
            if position >= cast(u64) strings.count {
                input.failed = true;
                return result;
            }

            source := Line_Wire_Input.{ data = strings, cursor = cast(s64) position, limit = strings.count };

            result.text = line_wire_string(*source);
            if source.failed input.failed = true;

            return result;
        case 0x0f;
            result.number = line_wire_uleb(input);
            return result;
        case 0x10;
            count = 4;
            result.reference = true;
        case 0x11;
            count = 1;
            relative = true;
        case 0x12;
            count = 2;
            relative = true;
        case 0x13;
            count = 4;
            relative = true;
        case 0x14;
            count = 8;
            relative = true;
        case 0x15;
            result.number = line_wire_uleb(input);
            relative = true;
        case 0x19;
            result.number = 1;
            return result;
        case;
            input.failed = true;
            return result;
    }

    if block {
        result.block_value = true;
        if count < 0 || count > input.limit - input.cursor input.failed = true;
        else input.cursor += count;
        return result;
    }

    if count result.number = line_wire_uint(input, count);

    if relative {
        result.reference = true;
        if result.number > 0xffffffff - cast(u64) cu_offset input.failed = true;
        else result.number += cast(u64) cu_offset;
    }

    return result;
}

dwarf_assign_attribute :: (die: *Dwarf_Die, name: u64, value: Dwarf_Value) -> bool {
    n := value.number;
    mask := dwarf_attribute_mask(name);

    if mask {
        text := name == 0x03 || name == 0x25 || name == 0x6e || name == 0x2007;
        if text != value.text_value || value.block_value || value.negative return false;
        if value.address && name != 0x11 && name != 0x12 return false;
        if (name == 0x11 && !value.address) || (name != 0x49 && value.reference) return false;
        if (name == 0x3f || name == 0x6a) && n > 1 return false;

        die.present |= mask;
    }

    if name == {
        case 0x03; die.name = value.text;
        case 0x25; die.producer = value.text;
        case 0x13;
            if n > 0xffff return false;

            die.language = n;
        case 0x10;
            die.statement_list = n;
            die.has_statement_list = true;
        case 0x11; die.low_pc = n;
        case 0x12;
            die.high_pc = n;
            die.high_relative = !value.address;
        case 0x0b; die.size = n;
        case 0x3e; die.encoding = n;
        case 0x3a;
            if n > 0xffffffff return false;

            die.decl_file = cast(u32) n;
        case 0x3b;
            if n > 0xffffffff return false;

            die.decl_line = cast(u32) n;
        case 0x49;
            if !value.reference || n > 0xffffffff return false;

            die.type_id = cast(u32) n;
        case 0x3f; die.external = n != 0;
        case 0x6a; die.is_main = n != 0;
        case 0x36;
            if n > 255 return false;

            die.calling_convention = cast(u8) n;
        case 0x6e, 0x2007; die.linkage_name = value.text;
        case 0x31;
            if !value.reference return false;

            die.origin = n;
        case 0x47;
            if !value.reference return false;

            die.specification = n;
        // Discontiguous ranges require .debug_ranges, absent from this API.
        case 0x55; return false;
    }

    return true;
}

dwarf_attribute_mask :: (name: u64) -> u64 {
    if name == {
        case 0x03; return 1;
        case 0x6e, 0x2007; return 2;
        case 0x25; return 4;
        case 0x13; return 8;
        case 0x10; return 16;
        case 0x11; return 32;
        case 0x12; return 64;
        case 0x0b; return 128;
        case 0x3e; return 256;
        case 0x3a; return 512;
        case 0x3b; return 1024;
        case 0x49; return 2048;
        case 0x3f; return 4096;
        case 0x6a; return 8192;
        case 0x36; return 16384;
    }

    return 0;
}

dwarf_resolve_dies :: (dies: [] Dwarf_Die) -> bool {
    // Resolve dependencies iteratively, without using the native call stack.
    for *die: dies {
        for *candidate: dies {
            if die.origin && cast(u64) candidate.offset == die.origin die.origin_die = candidate;
            if die.specification && cast(u64) candidate.offset == die.specification die.specification_die = candidate;
        }

        if (die.origin && !die.origin_die) || (die.specification && !die.specification_die) return false;
    }

    for 0..dies.count - 1 {
        pending: s64;
        progress := false;

        for *die: dies {
            if die.resolved continue;

            pending += 1;
            sources := (*Dwarf_Die).[die.origin_die, die.specification_die];
            ready := true;

            for source: sources if source && !source.resolved ready = false;
            if !ready continue;

            for source: sources {
                if !source continue;

                inherited := source.present & ~die.present;
                if inherited & 1 die.name = source.name;
                if inherited & 2 die.linkage_name = source.linkage_name;
                if inherited & 128 die.size = source.size;
                if inherited & 256 die.encoding = source.encoding;
                if inherited & 512 die.decl_file = source.decl_file;
                if inherited & 1024 die.decl_line = source.decl_line;
                if inherited & 2048 die.type_id = source.type_id;
                if inherited & 4096 die.external = source.external;
                if inherited & 8192 die.is_main = source.is_main;
                if inherited & 16384 die.calling_convention = source.calling_convention;

                die.present |= inherited & 32643;
            }

            if die.high_relative {
                if die.high_pc > 0xffffffffffffffff - die.low_pc return false;
                die.high_pc += die.low_pc;
            }

            if die.high_pc && die.high_pc < die.low_pc return false;

            die.resolved = true;
            progress = true;
        }

        if !pending return true;
        if !progress return false;
    }

    return true;
}

dwarf_is_type :: (tag: Dwarf_Tag) -> bool {
    n := cast(u64) tag;
    return n == 0x01 || n == 0x02 || n == 0x04 || n == 0x0f || n == 0x10 || n == 0x12 || n == 0x13 || n == 0x15 || n == 0x16 || n == 0x17 ||
        n == 0x1f || n == 0x20 || n == 0x21 || n == 0x24 || n == 0x26 || n == 0x27 || n == 0x29 || n == 0x2d || n == 0x31 || n == 0x35 ||
        n == 0x37 || n == 0x38 || n == 0x3b || n == 0x42 || n == 0x47 || n == 0x4a;
}

dwarf_project_lines :: (table: Line_Wire_Table, output: *Line_Info) -> bool {
    for directory: table.dirs array_add(*output.dirs, directory);
    for file: table.files array_add(*output.files, File_Info.{ dir_id = file.directory, path = file.path });

    sequence: *Line_Info_Sequence;

    for row: table.rows {
        if row.op_index return false;

        if !sequence {
            array_add(*output.sequences, Line_Info_Sequence.{});
            sequence = *output.sequences[output.sequences.count - 1];
        }

        if sequence.addresses.count && row.address < sequence.addresses[sequence.addresses.count - 1] return false;

        array_add(*sequence.addresses, row.address);
        array_add(*sequence.mappings, Line_Mapping.{ file_id = cast(Dwarf_File_Id) row.file, line = cast(u32) row.line,
            column = cast(u32) row.column, is_end = row.is_end, flags = cast(Line_Mapping.Flags) row.flags });
        if row.is_end sequence = null;
    }

    return sequence == null;
}

dwarf_release_unit :: (unit: *Compilation_Unit) {
    for *sub: unit.subprograms {
        for *block: sub.all_blocks {
            array_free(block.declarations);
            array_free(block.children);
        }

        BA.bucket_array_reset(*sub.all_blocks);
        array_free(sub.all_blocks.all_buckets);
        array_free(sub.all_blocks.unfull_buckets);
    }

    BA.bucket_array_reset(*unit.subprograms);
    array_free(unit.subprograms.all_buckets);
    array_free(unit.subprograms.unfull_buckets);
    array_free(unit.types);
    array_free(unit.globals);

    for sequence: unit.line_info.sequences {
        array_free(sequence.addresses);
        array_free(sequence.mappings);
    }

    array_free(unit.line_info.sequences);
    array_free(unit.line_info.files);
    array_free(unit.line_info.dirs);
    unit.* = .{};
}

dwarf_release_info :: (info: *Debug_Info) {
    for *unit: info.compilation_units dwarf_release_unit(unit);
    array_free(info.compilation_units);
    info.* = .{};
}
