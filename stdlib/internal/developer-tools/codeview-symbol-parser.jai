parse_codeview_symbols :: (info: *Codeview_Debug_Info, data: [] u8, verbose := false) -> success: bool {
    staged: Codeview_Debug_Info;
    committed := false;

    defer if !committed {
        cv_release_debug_symbols(*staged);
        log_error("Debug_Info: invalid or unsupported CodeView C13 symbols\n");
    }

    if !info || !cv_slice_valid(data.count, data.data) || !cv_symbol_target_valid(info.*) return false;

    input := Line_Wire_Input.{ data = data, limit = data.count };
    if cv_u32(*input) != 4 || input.failed return false;

    while input.cursor < input.limit && !input.failed {
        section := cv_u32(*input);
        length := cast(s64) cv_u32(*input);

        if input.failed || length > input.limit - input.cursor return false;

        after := input.cursor + length;
        subsection := Line_Wire_Input.{ data = .{ length, data.data + input.cursor }, limit = length };

        if !(section & 0x80000000) {
            if cast(Codeview_Symbol_Section) section == {
                case .SYMBOLS; if !cv_read_symbols(*staged, *subsection, verbose) return false;
                case .LINES; if !cv_read_lines(*staged, *subsection) return false;
                case .STRING_TABLE;
                    if staged.string_table.count || length == 0 || subsection.data[0] != 0 || subsection.data[length - 1] != 0 return false;

                    staged.string_table = .{ length, cast(*u8) alloc(length) };
                    if !staged.string_table.data return false;

                    memcpy(staged.string_table.data, subsection.data.data, length);
                case .FILE_CHECKSUMS;
                    if staged.file_checksums.count || !cv_read_checksums(*staged, *subsection) return false;
                case .INLINEELINES; if !cv_read_inlinees(*staged, *subsection) return false;
                case; return false;
            }
        }

        input.cursor = after;
        padding := (4 - (after % 4)) % 4;
        if padding > input.limit - input.cursor return false;
        for 0..padding - 1 if cv_u8(*input) != 0 return false;
    }

    if input.failed || (info.string_table.count && staged.string_table.count) return false;

    strings := staged;
    if !strings.string_table.count strings.string_table = info.string_table;

    for checksum: staged.file_checksums {
        for previous: info.file_checksums if previous.id == checksum.id return false;
        filename, ok := get_string_from_table(strings, checksum.filename_id);
        if !ok || !filename.count return false;
    }

    for sequence: staged.line_info for block : sequence.file_blocks {
        if !cv_checksum_present(staged, info.*, block.file_id) return false;
    }

    for inlinee: staged.inlinee_line_info {
        for previous: info.inlinee_line_info if previous.inlinee == inlinee.inlinee return false;
        if !cv_checksum_present(staged, info.*, inlinee.file_id) return false;
        for file: inlinee.extra_files if !cv_checksum_present(staged, info.*, cast(Codeview_File_Checksum_Id) file) return false;
    }

    for symbol: staged.symbols array_add(*info.symbols, symbol);
    for checksum: staged.file_checksums array_add(*info.file_checksums, checksum);
    for sequence: staged.line_info array_add(*info.line_info, sequence);
    for inlinee: staged.inlinee_line_info array_add(*info.inlinee_line_info, inlinee);
    if staged.string_table.count info.string_table = staged.string_table;

    array_free(staged.symbols);
    array_free(staged.file_checksums);
    array_free(staged.line_info);
    array_free(staged.inlinee_line_info);
    committed = true;
    if verbose log("Debug_Info: parsed C13 procedure/inline symbols and source tables\n");

    return true;
}

#scope_module

cv_release_debug_symbols :: (info: *Codeview_Debug_Info) {
    for symbol: info.symbols {
        if is_procedure(symbol) array_free((cast(*Codeview_Symbol_Procedure) symbol).inlinesites);

        if symbol.symbol_type == .INLINESITE {
            site := cast(*Codeview_Symbol_Inline_Site) symbol;
            array_free(site.binary_annotations);
            array_free(site.children);
        }

        free(symbol);
    }

    array_free(info.symbols);
    for checksum: info.file_checksums free(checksum.checksum.data);

    array_free(info.file_checksums);
    for sequence: info.line_info cv_release_blocks(*sequence.file_blocks);

    array_free(info.line_info);
    for inlinee: info.inlinee_line_info free(inlinee.extra_files.data);

    array_free(info.inlinee_line_info);
    free(info.string_table.data);
    info.string_table = .{};
}

#scope_file
#import "Basic";

Cv_Procedure_Link :: struct {
    offset: u32;
    next: u32;
}

Cv_Symbol_Scope :: struct {
    kind: Codeview_Symbol_Type;
    offset: u32;
    end: u32;
    procedure: *Codeview_Symbol_Procedure;
    site: *Codeview_Symbol_Inline_Site;
}

cv_symbol_target_valid :: (info: Codeview_Debug_Info) -> bool {
    return cv_slice_valid(info.symbols.count, info.symbols.data) && cv_slice_valid(info.line_info.count, info.line_info.data) &&
        cv_slice_valid(info.file_checksums.count, info.file_checksums.data) && cv_slice_valid(info.inlinee_line_info.count, info.inlinee_line_info.data) &&
        cv_slice_valid(info.string_table.count, info.string_table.data);
}

cv_checksum_present :: (first: Codeview_Debug_Info, second: Codeview_Debug_Info, id: Codeview_File_Checksum_Id) -> bool {
    for checksum: first.file_checksums if checksum.id == id return true;
    for checksum: second.file_checksums if checksum.id == id return true;
    return false;
}

cv_symbol_new :: (input: *Line_Wire_Input, info: *Codeview_Debug_Info, $T: Type, kind: Codeview_Symbol_Type) -> *T {
    result := cast(*T) alloc(size_of(T) + input.data.count);
    if !result return null;

    Initialize(result);
    result.base.symbol_type = kind;
    tail := cast(*u8) result + size_of(T);

    memcpy(tail, input.data.data, input.data.count);
    input.data.data = tail;
    array_add(*info.symbols, cast(*Codeview_Symbol) result);

    return result;
}

cv_zero_tail :: (input: *Line_Wire_Input, maximum: s64) -> bool {
    if input.failed || input.limit - input.cursor > maximum return false;
    while input.cursor < input.limit if cv_u8(input) != 0 return false;
    return !input.failed;
}

cv_read_symbols :: (info: *Codeview_Debug_Info, input: *Line_Wire_Input, verbose: bool) -> bool {
    scopes: [..] Cv_Symbol_Scope;
    links: [..] Cv_Procedure_Link;

    defer array_free(links);

    defer array_free(scopes);
    procedure: *Codeview_Symbol_Procedure;

    while input.cursor < input.limit && !input.failed {
        record_offset := input.cursor;
        length := cast(s64) cv_u16(input);

        if input.failed || length < 2 || length > input.limit - input.cursor return false;

        kind := cast(Codeview_Symbol_Type) cv_u16(input);
        after := input.cursor + length - 2;
        record := Line_Wire_Input.{ data = .{ length - 2, input.data.data + input.cursor }, limit = length - 2 };

        if kind == {
            case .LPROC32, .GPROC32, .LPROC32_ID, .GPROC32_ID;
                if procedure || scopes.count return false;

                result := cv_symbol_new(*record, info, Codeview_Symbol_Procedure, kind);
                if !result return false;

                result.parent = cv_u32(*record);
                result.end = cv_u32(*record);
                result.next = cv_u32(*record);
                result.length = cv_u32(*record);
                result.debug_start_offset = cv_u32(*record);
                result.debug_end_offset = cv_u32(*record);
                result.type = cv_id(*record);
                result.offset = cv_u32(*record);
                result.segment_index = cv_u16(*record);
                result.flags = cast(Codeview_Symbol_Procedure.Flags) cv_u8(*record);
                result.name = line_wire_string(*record);
                if record.failed || result.parent || (result.end && (result.end >= input.limit || result.end < record_offset)) ||
                    (result.next && (result.next >= input.limit || result.next <= record_offset)) || cast(u64) result.offset + result.length > 0x100000000 ||
                    result.debug_start_offset > result.length || result.debug_end_offset > result.length return false;

                procedure = result;
                array_add(*links, Cv_Procedure_Link.{ offset = cast(u32) record_offset, next = result.next });
                array_add(*scopes, Cv_Symbol_Scope.{ kind = kind, offset = cast(u32) record_offset, end = result.end, procedure = result });
            case .INLINESITE;
                if !procedure return false;

                result := cv_symbol_new(*record, info, Codeview_Symbol_Inline_Site, kind);
                if !result return false;

                result.parent = cv_u32(*record);
                result.end = cv_u32(*record);
                result.inlinee = cast(Item_Id) cv_u32(*record);
                if (result.parent && result.parent != scopes[scopes.count - 1].offset) ||
                    (result.end && (result.end >= input.limit || result.end < record_offset)) return false;
                if !cv_read_annotations(*record, *result.binary_annotations) return false;

                parent: *Codeview_Symbol_Inline_Site;
                for scope: scopes if scope.site parent = scope.site;
                if parent array_add(*parent.children, result);
                else array_add(*procedure.inlinesites, result);

                array_add(*scopes, Cv_Symbol_Scope.{ kind = kind, offset = cast(u32) record_offset, end = result.end, site = result });
            case .COMPILE3;
                result := cv_symbol_new(*record, info, Codeview_Symbol_Compile3, kind);
                if !result return false;

                result.language = cv_u8(*record);
                result.flags1 = cv_u8(*record);
                result.flags2 = cv_u8(*record);
                result.reserved = cv_u8(*record);
                result.target_cpu = cv_u16(*record);
                result.fe_version = .{ cv_u16(*record), cv_u16(*record), cv_u16(*record), cv_u16(*record) };
                result.be_version = .{ cv_u16(*record), cv_u16(*record), cv_u16(*record), cv_u16(*record) };
                result.compiler_version = line_wire_string(*record);
            case .COMPILE;
                result := cv_symbol_new(*record, info, Codeview_Symbol_Compile, kind);
                if !result return false;

                result.target_cpu = cv_u8(*record);
                result.language = cv_u8(*record);
                result.flags = cv_u16(*record);
                result.compiler_version = line_wire_string(*record);
            case .BLOCK32;
                if !procedure return false;

                parent := cv_u32(*record);
                end := cv_u32(*record);
                size := cv_u32(*record);
                start := cv_u32(*record);

                cv_u16(*record);
                line_wire_string(*record);
                if (parent && parent != scopes[scopes.count - 1].offset) || (end && (end >= input.limit || end < record_offset)) ||
                    cast(u64) start + size > 0x100000000 return false;

                array_add(*scopes, Cv_Symbol_Scope.{ kind = kind, offset = cast(u32) record_offset, end = end });
            case .END, .PROC_ID_END, .INLINESITE_END;
                if !scopes.count return false;

                top := scopes[scopes.count - 1];
                if kind == .INLINESITE_END && !top.site return false;
                if kind != .INLINESITE_END && top.site return false;
                if kind == .PROC_ID_END && !top.procedure return false;
                if top.end && top.end != record_offset return false;

                scopes.count -= 1;
                if top.procedure procedure = null;
            case;
                if !cv_consume_aux_symbol(*record, kind) return false;
                if verbose log("Debug_Info: validated unprojected auxiliary symbol %\n", kind);
        }

        if !cv_zero_tail(*record, 3) return false;

        input.cursor = after;
    }

    for link: links {
        if !link.next continue;
        found := false;
        for other: links if other.offset == link.next found = true;
        if !found return false;
    }

    return !input.failed && scopes.count == 0;
}

cv_annotation_uint :: (input: *Line_Wire_Input) -> u32 {
    first := cast(u32) cv_u8(input);
    if first < 0x80 return first;
    if first < 0xc0 return ((first & 0x3f) << 8) | cv_u8(input);

    if first < 0xe0 {
        result := (first & 0x1f) << 24;
        result |= (cast(u32) cv_u8(input)) << 16;
        result |= (cast(u32) cv_u8(input)) << 8;
        result |= cv_u8(input);

        return result;
    }

    input.failed = true;

    return 0;
}

cv_annotation_signed :: (value: u32) -> s32 {
    magnitude := cast(s32) (value >> 1);
    if value & 1 return -magnitude;
    return magnitude;
}

cv_read_annotations :: (input: *Line_Wire_Input, output: *[..] Codeview_Binary_Annotation) -> bool {
    while input.cursor < input.limit && !input.failed {
        opcode := cv_annotation_uint(input);
        if !opcode return cv_zero_tail(input, 3);
        if opcode > 13 return false;

        annotation: Codeview_Binary_Annotation;
        annotation.opcode = cast(Codeview_Binary_Annotation_Opcode) opcode;
        first := cv_annotation_uint(input);
        if opcode == 6 || opcode == 10 annotation.operand1.signed = cv_annotation_signed(first);
        else if opcode == 11 {
            annotation.operand1.unsigned = first & 15;
            annotation.operand2.signed = cv_annotation_signed(first >> 4);
        } else {
            annotation.operand1.unsigned = first;
            if opcode == 12 annotation.operand2.unsigned = cv_annotation_uint(input);
        }

        if input.failed return false;

        array_add(output, annotation);
    }

    return !input.failed;
}

cv_consume_aux_symbol :: (input: *Line_Wire_Input, kind: Codeview_Symbol_Type) -> bool {
    // These framed records have no reviewed common-base return storage. Their
    // wire payloads are validated, while the projected symbol graph consists
    // of the reviewed compiler/procedure/inline records above.
    if kind == {
        case .OBJNAME, .UDT;
            cv_u32(input);
            line_wire_string(input);
        case .BUILDINFO; cv_u32(input);
        case .FRAMEPROC;
            for 0..4 cv_u32(input);

            cv_u16(input);
            cv_u32(input);
        case .LOCAL;
            cv_u32(input);
            cv_u16(input);
            line_wire_string(input);
        case .REGREL32;
            cv_u32(input);
            cv_u32(input);
            cv_u16(input);
            line_wire_string(input);
        case .LDATA32, .GDATA32;
            cv_u32(input);
            cv_u32(input);
            cv_u16(input);
            line_wire_string(input);
        case .UNAMESPACE; line_wire_string(input);
        case .DEFRANGE_FRAMEPOINTER_REL_FULL_SCOPE; cv_u32(input);
        case .DEFRANGE_REGISTER, .DEFRANGE_FRAMEPOINTER_REL, .DEFRANGE_REGISTER_REL;
            cv_u32(input);
            if kind == .DEFRANGE_REGISTER_REL cv_u32(input);

            cv_u32(input);
            cv_u16(input);
            cv_u16(input);
            while input.limit - input.cursor >= 4 cv_u32(input);
        case .INLINEES, .CALLEES, .CALLERS;
            count := cast(s64) cv_u32(input);
            if input.failed || count > (input.limit - input.cursor) / 4 return false;
            for 0..count - 1 cv_u32(input);
            if kind != .INLINEES && input.limit - input.cursor >= count * 4 for 0..count - 1 cv_u32(input);
        case .CONSTANT;
            cv_u32(input);
            cv_numeric(input, true);
            line_wire_string(input);
        case; return false;
    }

    return !input.failed;
}

cv_read_checksums :: (info: *Codeview_Debug_Info, input: *Line_Wire_Input) -> bool {
    while input.cursor < input.limit && !input.failed {
        item: Codeview_File_Checksum;
        item.id = cast(Codeview_File_Checksum_Id) input.cursor;
        item.filename_id = cast(Codeview_String_Table_Id) cv_u32(input);
        length := cast(s64) cv_u8(input);
        kind := cv_u8(input);

        if input.failed || kind > 3 || length > input.limit - input.cursor return false;

        required := s64.[0, 16, 20, 32];
        if length != required[kind] return false;

        item.checksum_type = cast(Codeview_File_Checksum.Checksum_Type) kind;
        array_add(*info.file_checksums, item);
        current := *info.file_checksums[info.file_checksums.count - 1];

        if length {
            current.checksum = .{ length, cast(*u8) alloc(length) };
            if !current.checksum.data return false;
            memcpy(current.checksum.data, input.data.data + input.cursor, length);
            input.cursor += length;
        }

        padding := (4 - input.cursor % 4) % 4;
        if padding > input.limit - input.cursor return false;
        for 0..padding - 1 if cv_u8(input) != 0 return false;
    }

    return !input.failed;
}

cv_read_inlinees :: (info: *Codeview_Debug_Info, input: *Line_Wire_Input) -> bool {
    variant := cv_u32(input);
    if input.failed || variant > 1 return false;

    while input.cursor < input.limit && !input.failed {
        entry: Codeview_Inlinee_Line_Info;
        entry.inlinee = cv_id(input);
        entry.file_id = cast(Codeview_File_Checksum_Id) cv_u32(input);
        entry.source_line = cv_u32(input);
        if input.failed return false;
        for previous: info.inlinee_line_info if previous.inlinee == entry.inlinee return false;

        array_add(*info.inlinee_line_info, entry);
        current := *info.inlinee_line_info[info.inlinee_line_info.count - 1];

        if variant == 1 {
            count := cast(s64) cv_u32(input);
            if input.failed || count > (input.limit - input.cursor) / 4 return false;

            if count {
                current.extra_files = .{ count, cast(*u32) alloc(count * 4) };
                if !current.extra_files.data return false;
                for *file: current.extra_files file.* = cv_u32(input);
            }
        }
    }

    return !input.failed;
}

cv_read_lines :: (info: *Codeview_Debug_Info, input: *Line_Wire_Input) -> bool {
    sequence: Codeview_Line_Info_Sequence;
    sequence.text_offset = cv_u32(input);
    sequence.segment_index = cv_u16(input);
    flags := cv_u16(input);

    sequence.flags = cast(Codeview_Line_Info_Sequence.Flags) flags;
    sequence.length_in_bytes = cv_u32(input);
    if input.failed || flags > 1 || cast(u64) sequence.text_offset + sequence.length_in_bytes > 0x100000000 return false;

    array_add(*info.line_info, sequence);
    stored := *info.line_info[info.line_info.count - 1];

    while input.cursor < input.limit && !input.failed {
        block_start := input.cursor;
        block: Codeview_Lines_File_Block;
        block.file_id = cast(Codeview_File_Checksum_Id) cv_u32(input);
        count := cast(s64) cv_u32(input);
        size := cast(s64) cv_u32(input);
        stride: s64 = 8;

        if flags stride = 12;
        if input.failed || count > (input.limit - input.cursor) / stride || size != 12 + count * stride || size > input.limit - block_start return false;

        array_add(*stored.file_blocks, block);
        current := *stored.file_blocks[stored.file_blocks.count - 1];

        if count {
            current.lines = .{ count, cast(*Codeview_Line_Info) alloc(count * size_of(Codeview_Line_Info)) };
            if !current.lines.data return false;

            for *line, index: current.lines {
                line.text_offset = cv_u32(input);
                line.line_and_delta_and_flag = cv_u32(input);
                if line.text_offset > sequence.length_in_bytes || (index && line.text_offset < current.lines[index - 1].text_offset) return false;
            }

            if flags {
                current.columns = .{ count, cast(*Codeview_Column_Type) alloc(count * size_of(Codeview_Column_Type)) };
                if !current.columns.data return false;

                for *column: current.columns {
                    column.* = cv_u16(input);
                    cv_u16(input);
                }
            }
        }
    }

    return !input.failed;
}
