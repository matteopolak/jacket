decode_cfa_instructions :: (data: [] u8) -> [] Call_Frame_Address_Instruction, success: bool {
    input: Frame_Input = .{ data = data };
    rows: [..] Call_Frame_Address_Instruction;

    while input.position < data.count && !input.failed {
        opcode := frame_uint(*input, 1);
        row: Call_Frame_Address_Instruction;
        primary := opcode & 0xc0;

        if primary != 0 {
            row.type = cast(Call_Frame_Address_Instruction_Type) primary;
            row.arg1.uvalue = opcode & 0x3f;
            if primary == 0x80 row.arg2.uvalue = frame_uleb(*input);
        } else {
            row.type = cast(Call_Frame_Address_Instruction_Type) opcode;

            if opcode == 0 || opcode == 10 || opcode == 11 {
                // These instructions have no encoded arguments.
            } else if opcode == 2 row.arg1.uvalue = frame_uint(*input, 1);
            else if opcode == 3 row.arg1.uvalue = frame_uint(*input, 2);
            else if opcode == 4 row.arg1.uvalue = frame_uint(*input, 4);
            else if opcode == 5 || opcode == 9 || opcode == 12 || opcode == 19 {
                row.arg1.uvalue = frame_uleb(*input);
                row.arg2.uvalue = frame_uleb(*input);
            } else if opcode == 6 || opcode == 7 || opcode == 8 || opcode == 13 || opcode == 14 row.arg1.uvalue = frame_uleb(*input);
            else if opcode == 16 || opcode == 17 || opcode == 20 {
                row.arg1.uvalue = frame_uleb(*input);
                row.arg2.svalue = frame_sleb(*input);
            } else if opcode == 18 row.arg1.svalue = frame_sleb(*input);
            else input.failed = true;
        }

        if !input.failed array_add(*rows, row);
    }

    if input.failed {
        array_free(rows);
        return .{}, false;
    }

    return .{ rows.count, rows.data }, true;
}

parse_common_information_entry :: (data: [] u8, offset_in: s64) -> success: bool, Frame_Common_Information_Entry, offset: s64 {
    empty: Frame_Common_Information_Entry;
    if offset_in < 0 || offset_in > data.count return false, empty, offset_in;

    input: Frame_Input = .{ data = data, position = offset_in };
    length := frame_uint(*input, 4);

    if length == 0xffffffff length = frame_uint(*input, 8);
    else if length >= 0xfffffff0 return false, empty, offset_in;

    if input.failed || length == 0 || length > cast(u64) (data.count - input.position) return false, empty, offset_in;

    end := input.position + cast(s64) length;

    input.data.count = end;
    result: Frame_Common_Information_Entry;
    result.length = length;
    result.offset = cast(u64) offset_in;
    result.id = cast(u32) frame_uint(*input, 4);
    if result.id != 0 && result.id != 0xffffffff return false, empty, offset_in;

    result.version = cast(u8) frame_uint(*input, 1);
    if result.version != 1 && result.version != 3 && result.version != 4 return false, empty, offset_in;
    if result.id == 0 && result.version != 1 return false, empty, offset_in;

    start := input.position;

    while input.position < end && input.data[input.position] != 0 input.position += 1;
    if input.position == end return false, empty, offset_in;

    result.augmentation = .{ input.position - start, input.data.data + start };
    input.position += 1;

    if result.version == 4 {
        address_size := frame_uint(*input, 1);
        segment_size := frame_uint(*input, 1);
        if (address_size != 4 && address_size != 8) || segment_size != 0 return false, empty, offset_in;
    }

    result.code_alignment_factor = frame_uleb(*input);
    result.data_alignment_factor = frame_sleb(*input);
    if result.version == 1 result.return_address_register = frame_uint(*input, 1);
    else result.return_address_register = frame_uleb(*input);

    if result.augmentation.count > 0 {
        if result.augmentation[0] != #char "z" return false, empty, offset_in;

        size := frame_uleb(*input);
        if input.failed || size > cast(u64) (end - input.position) return false, empty, offset_in;

        result.augmentation_data = .{ cast(s64) size, input.data.data + input.position };
        input.position += cast(s64) size;
    }

    if input.failed || result.code_alignment_factor == 0 return false, empty, offset_in;

    instructions, ok := decode_cfa_instructions(.{ end - input.position, input.data.data + input.position });
    if !ok return false, empty, offset_in;

    result.initial_instructions = instructions;

    return true, result, end;
}

parse_frame_description_entry :: (cie: Frame_Common_Information_Entry, frame_section_address: u64, data: [] u8, offset_in: s64) -> success: bool, Frame_Description_Entry, offset: s64 {
    empty: Frame_Description_Entry;
    if data.count < 0 || offset_in < 0 || offset_in > data.count return false, empty, offset_in;
    if cast(u64) data.count > 0xffffffffffffffff - frame_section_address return false, empty, offset_in;
    if cie.id == 0 return frame_eh_entry(cie, frame_section_address, data, offset_in);
    // This supported path is DWARF32 .debug_frame v4 with explicit address size.
    // Its absolute CIE pointer has the same encoded and resolved section offset.
    if cie.id != 0xffffffff || cie.version != 4 || cie.augmentation.count != 0 || cie.offset > cast(u64) data.count return false, empty, offset_in;

    common: Frame_Input = .{ data = data, position = cast(s64) cie.offset };
    common_length := frame_uint(*common, 4);

    if common.failed || common_length < 8 || common_length >= 0xfffffff0 || common_length != cie.length || common_length > cast(u64) (data.count - common.position) return false, empty, offset_in;

    common_end := common.position + cast(s64) common_length;

    common.data.count = common_end;
    if frame_uint(*common, 4) != 0xffffffff || frame_uint(*common, 1) != 4 || frame_uint(*common, 1) != 0 return false, empty, offset_in;

    address_size := frame_uint(*common, 1);
    segment_size := frame_uint(*common, 1);

    if common.failed || (address_size != 4 && address_size != 8) || segment_size != 0 return false, empty, offset_in;

    input: Frame_Input = .{ data = data, position = offset_in };
    length := frame_uint(*input, 4);

    if input.failed || length < 4 + 2 * address_size || length >= 0xfffffff0 || length > cast(u64) (data.count - input.position) return false, empty, offset_in;

    end := input.position + cast(s64) length;
    if cast(s64) cie.offset < end && common_end > offset_in return false, empty, offset_in;

    input.data.count = end;
    result: Frame_Description_Entry;
    result.length = length;
    result.cie_offset = frame_uint(*input, 4);
    if result.cie_offset != cie.offset return false, empty, offset_in;

    result.pc_begin = frame_uint(*input, cast(s64) address_size);
    result.pc_range = frame_uint(*input, cast(s64) address_size);
    if input.failed return false, empty, offset_in;

    instructions, ok := decode_cfa_instructions(.{ end - input.position, data.data + input.position });
    if !ok return false, empty, offset_in;

    result.instructions = instructions;

    return true, result, end;
}

#scope_file
#import "Basic";

frame_eh_entry :: (cie: Frame_Common_Information_Entry, section_address: u64, data: [] u8, at: s64) -> bool, Frame_Description_Entry, s64 {
    empty: Frame_Description_Entry;
    if cie.version != 1 || cie.offset > cast(u64) data.count return false, empty, at;

    encoding, augmented, encoding_ok := frame_eh_encoding(cie);
    if !encoding_ok return false, empty, at;

    input: Frame_Input = .{ data = data, position = at };
    length := frame_uint(*input, 4);
    width: s64 = ifx encoding == 0 then 8 else 4;

    if input.failed || length < cast(u64) (4 + width * 2) || length >= 0xfffffff0 || length > cast(u64) (data.count - input.position) return false, empty, at;

    end := input.position + cast(s64) length;

    input.data.count = end;
    pointer_field := input.position;
    distance := frame_uint(*input, 4);

    if input.failed || distance > cast(u64) pointer_field return false, empty, at;

    resolved := cast(u64) pointer_field - distance;
    if resolved != cie.offset return false, empty, at;

    result: Frame_Description_Entry;
    result.length = length;
    result.cie_offset = resolved;
    address_field := input.position;
    encoded_address := frame_uint(*input, width);

    if input.failed return false, empty, at;

    if encoding == 0 result.pc_begin = encoded_address;
    else {
        delta := cast(s64) encoded_address;
        if encoded_address & 0x80000000 delta -= 0x100000000;

        base := section_address + cast(u64) address_field;

        if delta < 0 {
            magnitude := cast(u64) -delta;
            if magnitude > base return false, empty, at;
            result.pc_begin = base - magnitude;
        } else {
            magnitude := cast(u64) delta;
            if magnitude > 0xffffffffffffffff - base return false, empty, at;
            result.pc_begin = base + magnitude;
        }
    }

    result.pc_range = frame_uint(*input, width);

    if augmented {
        bytes := frame_uleb(*input);
        if input.failed || bytes > cast(u64) (end - input.position) return false, empty, at;
        result.augmentation_data = .{ cast(s64) bytes, data.data + input.position };
        input.position += cast(s64) bytes;
    }

    if input.failed return false, empty, at;

    instructions, ok := decode_cfa_instructions(.{ end - input.position, data.data + input.position });
    if !ok return false, empty, at;

    result.instructions = instructions;

    return true, result, end;
}

frame_eh_encoding :: (cie: Frame_Common_Information_Entry) -> u8, bool, bool {
    if cie.augmentation.count < 0 || cie.augmentation_data.count < 0 return 0, false, false;
    if cie.augmentation.count == 0 return 0, false, cie.augmentation_data.count == 0;
    if cie.augmentation[0] != #char "z" return 0, false, false;

    encoding: u8;
    consumed: s64;

    for index: 1..cie.augmentation.count - 1 {
        letter := cie.augmentation[index];
        if letter == #char "R" {
            if consumed != 0 || cie.augmentation_data.count < 1 return 0, false, false;
            encoding = cie.augmentation_data[0];
            consumed = 1;
        } else if letter != #char "S" return 0, false, false;
    }

    if consumed != cie.augmentation_data.count || (encoding != 0 && encoding != 0x1b) return 0, false, false;

    return encoding, true, true;
}

Frame_Input :: struct {
    data: [] u8;
    position: s64;
    failed: bool;
}

frame_uint :: (input: *Frame_Input, count: s64) -> u64 {
    if count < 1 || count > 8 || input.position < 0 || input.position > input.data.count || count > input.data.count - input.position {
        input.failed = true;
        return 0;
    }

    value: u64;
    for index: 0..count - 1 value |= (cast(u64) input.data[input.position + index]) << (index * 8);

    input.position += count;

    return value;
}

frame_uleb :: (input: *Frame_Input) -> u64 {
    value: u64;

    for index: 0..9 {
        byte := frame_uint(input, 1);

        if input.failed return 0;

        payload := byte & 0x7f;
        if index == 9 && payload > 1 {
            input.failed = true;
            return 0;
        }

        value |= payload << (index * 7);
        if !(byte & 0x80) return value;
    }

    input.failed = true;

    return 0;
}

frame_sleb :: (input: *Frame_Input) -> s64 {
    value: u64;

    for index: 0..9 {
        byte := frame_uint(input, 1);

        if input.failed return 0;

        payload := byte & 0x7f;
        if index == 9 && payload != 0 && payload != 0x7f {
            input.failed = true;
            return 0;
        }

        value |= payload << (index * 7);

        if !(byte & 0x80) {
            shift := (index + 1) * 7;
            if shift < 64 && byte & 0x40 value |= 0xffffffffffffffff << shift;
            return cast(s64) value;
        }
    }

    input.failed = true;

    return 0;
}
