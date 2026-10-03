// Independently authored from DWARF4 sections6.2 and7.21.
// Staged parser dependency; raw encoded file/directory IDs are deliberately retained.
#scope_module

Line_Wire_File :: struct {
    path: string;
    directory: u64;
    modification_time: u64;
    size: u64;
}

Line_Wire_Row :: struct {
    address: u64;
    op_index: u64;
    file: u64 = 1;
    line: s64 = 1;
    column: u64;
    flags: u8;
    isa: u64;
    discriminator: u64;
    is_end: bool;
}

Line_Wire_Table :: struct {
    header: D.Line_Program_Header;
    dirs: [..] string;
    files: [..] Line_Wire_File;
    rows: [..] Line_Wire_Row;
    next_offset: s64;
}

Line_Wire_Input :: struct {
    data: [] u8;
    cursor: s64;
    limit: s64;
    failed: bool;
}

line_wire_release :: (table: *Line_Wire_Table) {
    array_free(table.dirs);
    array_free(table.files);
    array_free(table.rows);
    table.* = .{};
}

line_wire_decode :: (data: [] u8, offset: s64, address_size: s64) -> Line_Wire_Table, bool {
    empty: Line_Wire_Table;
    if data.count < 0 || !data.data || offset < 0 || offset > data.count || (address_size != 4 && address_size != 8) return empty, false;

    input: Line_Wire_Input = .{ data = data, cursor = offset, limit = data.count };
    result: Line_Wire_Table;
    success := false;

    defer if !success line_wire_release(*result);
    result.header.size = cast(u32) line_wire_uint(*input, 4);
    if input.failed || result.header.size >= 0xfffffff0 || result.header.size > input.limit - input.cursor return empty, false;

    input.limit = input.cursor + result.header.size;
    result.next_offset = input.limit;
    result.header.version = cast(u16) line_wire_uint(*input, 2);
    if result.header.version != 4 return empty, false;

    result.header.header_length = cast(u32) line_wire_uint(*input, 4);
    if input.failed || result.header.header_length > input.limit - input.cursor return empty, false;

    program_end := input.limit;

    input.limit = input.cursor + result.header.header_length;
    result.header.minimum_instruction_length = cast(u8) line_wire_uint(*input, 1);
    result.header.maximum_operations_per_instruction = cast(u8) line_wire_uint(*input, 1);
    result.header.default_is_stmt = line_wire_uint(*input, 1) != 0;
    encoded_line_base := cast(u8) line_wire_uint(*input, 1);

    result.header.line_base = cast(s8) encoded_line_base;
    result.header.line_range = cast(u8) line_wire_uint(*input, 1);
    result.header.opcode_base = cast(u8) line_wire_uint(*input, 1);
    if input.failed || !result.header.minimum_instruction_length || !result.header.maximum_operations_per_instruction ||
        !result.header.line_range || !result.header.opcode_base return empty, false;

    lengths: [256] u8;
    for index: 1..cast(s64) result.header.opcode_base - 1 lengths[index] = cast(u8) line_wire_uint(*input, 1);

    known_lengths := u8.[0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 1];
    for length, index: known_lengths {
        if index + 1 >= result.header.opcode_base break;
        if lengths[index + 1] != length return empty, false;
    }

    while !input.failed {
        directory := line_wire_string(*input);
        if input.failed return empty, false;
        if !directory.count break;
        array_add(*result.dirs, directory);
    }

    while !input.failed {
        file := line_wire_file(*input);

        if input.failed return empty, false;
        if !file.path.count break;
        if file.directory > cast(u64) result.dirs.count return empty, false;

        array_add(*result.files, file);
    }

    if input.failed || input.cursor != input.limit return empty, false;

    input.limit = program_end;
    row := line_wire_initial(result.header.default_is_stmt);
    sequence_open := false;

    while input.cursor < input.limit && !input.failed {
        opcode := line_wire_uint(*input, 1);
        if opcode == 0 {
            length := line_wire_uleb(*input);
            available := cast(u64) (input.limit - input.cursor);

            if input.failed || !length || length > available return empty, false;

            after := input.cursor + cast(s64) length;

            input.limit = after;
            extended := line_wire_uint(*input, 1);
            if extended == 1 {
                row.is_end = true;
                if !line_wire_emit(*result, *row) return empty, false;
                row = line_wire_initial(result.header.default_is_stmt);
                sequence_open = false;
            } else if extended == 2 {
                row.address = line_wire_uint(*input, address_size);
                row.op_index = 0;
                sequence_open = true;
            } else if extended == 3 {
                file := line_wire_file(*input);
                if !file.path.count || file.directory > cast(u64) result.dirs.count return empty, false;
                array_add(*result.files, file);
            } else if extended == 4 row.discriminator = line_wire_uleb(*input);
            else return empty, false;

            if input.failed || input.cursor != after return empty, false;

            input.limit = program_end;
        } else if opcode >= result.header.opcode_base {
            adjusted := opcode - result.header.opcode_base;
            if !line_wire_advance(*row, result.header, adjusted / result.header.line_range) return empty, false;

            delta := cast(s64) (adjusted % result.header.line_range);
            delta += result.header.line_base;
            if !line_wire_add_line(*row, delta) || !line_wire_emit(*result, *row) return empty, false;

            sequence_open = true;
        } else {
            if opcode == {
                case 1;
                    if !line_wire_emit(*result, *row) return empty, false;

                    sequence_open = true;
                case 2;
                    advance := line_wire_uleb(*input);
                    if !line_wire_advance(*row, result.header, advance) return empty, false;

                    sequence_open = true;
                case 3;
                    delta := line_wire_sleb(*input);
                    if !line_wire_add_line(*row, delta) return empty, false;
                case 4; row.file = line_wire_uleb(*input);
                case 5; row.column = line_wire_uleb(*input);
                case 6; row.flags ^= 1;
                case 7; row.flags |= 2;
                case 8;
                    advance := (255 - cast(u64) result.header.opcode_base) / result.header.line_range;
                    if !line_wire_advance(*row, result.header, advance) return empty, false;

                    sequence_open = true;
                case 9;
                    advance := line_wire_uint(*input, 2);
                    if advance > 0xffffffffffffffff - row.address return empty, false;

                    row.address += advance;
                    row.op_index = 0;
                    sequence_open = true;
                case 10; row.flags |= 4;
                case 11; row.flags |= 8;
                case 12; row.isa = line_wire_uleb(*input);
                case;
                    // Unrecognized standard extensions have no defined state semantics here.
                    return empty, false;
            }
        }
    }

    if input.failed || sequence_open return empty, false;

    success = true;

    return result, true;
}

line_wire_initial :: (statement: bool) -> Line_Wire_Row {
    result: Line_Wire_Row;
    if statement result.flags = 1;
    return result;
}

line_wire_emit :: (table: *Line_Wire_Table, row: *Line_Wire_Row) -> bool {
    if !row.file || row.file > cast(u64) table.files.count || row.line < 0 || row.line > 0xffffffff || row.column > 0xffffffff return false;

    array_add(*table.rows, row.*);
    row.flags &= 1;
    row.discriminator = 0;

    return true;
}

line_wire_add_line :: (row: *Line_Wire_Row, delta: s64) -> bool {
    if delta > 0 && row.line > 0x7fffffffffffffff - delta return false;
    if delta < 0 && row.line < (-0x7fffffffffffffff - 1) - delta return false;
    row.line += delta;
    return true;
}

line_wire_advance :: (row: *Line_Wire_Row, header: D.Line_Program_Header, count: u64) -> bool {
    if count > 0xffffffffffffffff - row.op_index return false;

    operations := row.op_index + count;
    instructions := operations / header.maximum_operations_per_instruction;

    if instructions > (0xffffffffffffffff - row.address) / header.minimum_instruction_length return false;

    row.address += instructions * header.minimum_instruction_length;
    row.op_index = operations % header.maximum_operations_per_instruction;

    return true;
}

line_wire_uint :: (input: *Line_Wire_Input, count: s64) -> u64 {
    if input.failed || count < 0 || count > 8 || count > input.limit - input.cursor {
        input.failed = true;
        return 0;
    }

    result: u64;

    for index: 0..count - 1 {
        byte := cast(u64) input.data[input.cursor + index];
        result |= byte << (index * 8);
    }

    input.cursor += count;

    return result;
}

line_wire_uleb :: (input: *Line_Wire_Input) -> u64 {
    result: u64;

    for index: 0..9 {
        byte := line_wire_uint(input, 1);
        payload := byte & 0x7f;

        if input.failed || (index == 9 && payload > 1) break;

        result |= payload << (index * 7);
        if !(byte & 0x80) return result;
    }

    input.failed = true;

    return 0;
}

line_wire_sleb :: (input: *Line_Wire_Input) -> s64 {
    result: u64;

    for index: 0..9 {
        byte := line_wire_uint(input, 1);
        payload := byte & 0x7f;

        if input.failed || (index == 9 && payload != 0 && payload != 0x7f) break;

        result |= payload << (index * 7);

        if !(byte & 0x80) {
            shift := (index + 1) * 7;
            if shift < 64 && byte & 0x40 result |= 0xffffffffffffffff << shift;
            return cast(s64) result;
        }
    }

    input.failed = true;

    return 0;
}

line_wire_string :: (input: *Line_Wire_Input) -> string {
    start := input.cursor;

    while !input.failed {
        byte := line_wire_uint(input, 1);
        if !byte && !input.failed return .{ input.cursor - start - 1, input.data.data + start };
    }

    return "";
}

line_wire_file :: (input: *Line_Wire_Input) -> Line_Wire_File {
    result: Line_Wire_File;
    result.path = line_wire_string(input);
    if result.path.count {
        result.directory = line_wire_uleb(input);
        result.modification_time = line_wire_uleb(input);
        result.size = line_wire_uleb(input);
    }

    return result;
}

#scope_file
#import "Basic";
D :: #import "Debug_Info";
