decode_inline_site_line_info :: (info: Codeview_Debug_Info, procedure: Codeview_Symbol_Procedure, site: Codeview_Symbol_Inline_Site) -> [] Codeview_Line_Info_Sequence, success: bool {
    output: [..] Codeview_Line_Info_Sequence;
    keep := false;

    defer if !keep {
        cv_release_sequences(output);
        log_error("Debug_Info: invalid or unfinished inline annotation ranges\n");
    }

    if !cv_slice_valid(info.inlinee_line_info.count, info.inlinee_line_info.data) || !cv_slice_valid(site.binary_annotations.count, site.binary_annotations.data) return .{}, false;

    source: *Codeview_Inlinee_Line_Info;

    for *entry: info.inlinee_line_info {
        if cast(u32) entry.inlinee != cast(u32) site.inlinee continue;
        if source return .{}, false;
        source = entry;
    }

    if !source || !is_procedure(*procedure.base) || site.symbol_type != .INLINESITE || !procedure.length ||
        cast(u64) procedure.offset + procedure.length > 0x100000000 return .{}, false;

    state: Cv_Inline_State;
    state.line = source.source_line;
    state.file = source.file_id;
    offset: u64;
    base: u64;
    pending := false;
    pending_start: u64;
    pending_state: Cv_Inline_State;

    for annotation: site.binary_annotations {
        first := cast(u64) annotation.operand1.unsigned;
        move := false;
        finish := false;
        length: u64;
        line_delta: s64;

        if annotation.opcode == {
            case .CODE_OFFSET;
                offset = first;
                move = true;
            case .CHANGE_CODE_OFFSET_BASE;
                if pending return .{}, false;

                base = first;
            case .CHANGE_CODE_OFFSET;
                offset += first;
                move = true;
            case .CHANGE_CODE_OFFSET_AND_LINE_OFFSET;
                offset += first;
                line_delta = annotation.operand2.signed;
                move = true;
            case .CHANGE_CODE_LENGTH;
                if !pending return .{}, false;

                length = first;
                finish = true;
            case .CHANGE_CODE_LENGTH_AND_CODE_OFFSET;
                offset += annotation.operand2.unsigned;
                length = first;
                move = true;
                finish = true;
            case .CHANGE_FILE; state.file = cast(Codeview_File_Checksum_Id) first;
            case .CHANGE_LINE_OFFSET; state.line += annotation.operand1.signed;
            case .CHANGE_LINE_END_DELTA;
                if first > 127 return .{}, false;

                state.end_delta = cast(u8) first;
            case .CHANGE_RANGE_KIND;
                if first > 1 return .{}, false;

                state.statement = first == 1;
            case .CHANGE_COLUMN_START;
                if first > 0xffff return .{}, false;

                state.column = cast(u16) first;
            case .CHANGE_COLUMN_END, .CHANGE_COLUMN_END_DELTA;
            // The reviewed public line shape stores start columns only.
            case; return .{}, false;
        }

        if offset > 0xffffffff || base + offset > procedure.length || state.line < 0 || state.line > 0xffffff return .{}, false;

        if move {
            start := cast(u64) procedure.offset + base + offset;
            if pending && !cv_inline_emit(*output, procedure.segment_index, pending_start, start, pending_state) return .{}, false;

            state.line += line_delta;
            if state.line < 0 || state.line > 0xffffff return .{}, false;

            pending = true;
            pending_start = start;
            pending_state = state;
        }

        if finish {
            end := pending_start + length;
            if !length || end > cast(u64) procedure.offset + procedure.length || !cv_inline_emit(*output, procedure.segment_index, pending_start, end, pending_state) return .{}, false;
            pending = false;
        }
    }

    if pending || !output.count return .{}, false;

    keep = true;

    return .{ output.count, output.data }, true;
}

overlay_inlinesites :: (sequences: [] Codeview_Line_Info_Sequence, procedure: Codeview_Symbol_Procedure, inline_sites: [] *Codeview_Symbol_Inline_Site, debug_info: Codeview_Debug_Info) -> bool, [] Codeview_Line_Info_Sequence {
    owned: [..] Codeview_Line_Info_Sequence;
    keep := false;

    defer if !keep {
        cv_release_sequences(owned);
        log_error("Debug_Info: inline site could not be decoded or placed\n");
    }

    if !cv_slice_valid(sequences.count, sequences.data) || !cv_slice_valid(inline_sites.count, inline_sites.data) return false, .{};

    for sequence: sequences {
        clone, ok := cv_clone_sequence(sequence, sequence.text_offset, sequence.length_in_bytes);
        if !ok return false, .{};
        array_add(*owned, clone);
    }

    for site: inline_sites if !cv_overlay_site(*owned, procedure, site, debug_info, .{}, 0) return false, .{};

    keep = true;

    return true, .{ owned.count, owned.data };
}

overlay_inlinesite :: (sequences: [] Codeview_Line_Info_Sequence, procedure: Codeview_Symbol_Procedure, inline_site: *Codeview_Symbol_Inline_Site, debug_info: Codeview_Debug_Info) -> bool, new_sequences: [] Codeview_Line_Info_Sequence {
    sites: [1] *Codeview_Symbol_Inline_Site = .[inline_site];
    return overlay_inlinesites(sequences, procedure, sites, debug_info);
}

#scope_module

cv_slice_valid :: (count: s64, data: *void) -> bool {
    return count >= 0 && (count == 0 || data != null);
}

cv_release_sequences :: (sequences: [] Codeview_Line_Info_Sequence) {
    for sequence: sequences cv_release_blocks(*sequence.file_blocks);
    free(sequences.data);
}

Cv_Inline_State :: struct {
    file: Codeview_File_Checksum_Id;
    line: s64;
    column: u16;
    end_delta: u8;
    statement: bool = true;
}

cv_inline_emit :: (output: *[..] Codeview_Line_Info_Sequence, segment: u16, start: u64, end: u64, state: Cv_Inline_State) -> bool {
    if end <= start || end > 0x100000000 return false;

    sequence: *Codeview_Line_Info_Sequence;

    if output.count {
        last := *output.*[output.count - 1];
        last_end := cast(u64) last.text_offset + last.length_in_bytes;
        if start < last_end return false;
        if start == last_end sequence = last;
    }

    if !sequence {
        value: Codeview_Line_Info_Sequence;
        value.text_offset = cast(u32) start;
        value.segment_index = segment;
        value.flags = .HAS_COLUMNS;
        array_add(output, value);
        sequence = *output.*[output.count - 1];
    }

    block: *Codeview_Lines_File_Block;
    for *existing: sequence.file_blocks if existing.file_id == state.file block = existing;

    if !block {
        array_add(*sequence.file_blocks, Codeview_Lines_File_Block.{ file_id = state.file });
        block = *sequence.file_blocks[sequence.file_blocks.count - 1];
    }

    count := block.lines.count + 1;
    lines := cast(*Codeview_Line_Info) alloc(count * size_of(Codeview_Line_Info));

    if !lines return false;

    columns := cast(*Codeview_Column_Type) alloc(count * size_of(Codeview_Column_Type));
    if !columns {
        free(lines);
        return false;
    }

    memcpy(lines, block.lines.data, block.lines.count * size_of(Codeview_Line_Info));
    memcpy(columns, block.columns.data, block.columns.count * size_of(Codeview_Column_Type));
    flags := (cast(u32) state.line) | ((cast(u32) state.end_delta) << 24);
    if state.statement flags |= 0x80000000;

    lines[count - 1] = .{ text_offset = cast(u32) (start - sequence.text_offset), line_and_delta_and_flag = flags };
    columns[count - 1] = state.column;
    free(block.lines.data);
    free(block.columns.data);
    block.lines = .{ count, lines };
    block.columns = .{ count, columns };
    sequence.length_in_bytes = cast(u32) (end - sequence.text_offset);

    return true;
}

cv_clone_sequence :: (sequence: Codeview_Line_Info_Sequence, start: u32, length: u32) -> Codeview_Line_Info_Sequence, bool {
    result := sequence;
    result.text_offset = start;
    result.length_in_bytes = length;
    blocks, ok := cv_truncate_blocks(result, sequence.text_offset);
    if !ok return .{}, false;

    result.file_blocks = blocks;

    return result, true;
}

cv_overlay_site :: (owned: *[..] Codeview_Line_Info_Sequence, procedure: Codeview_Symbol_Procedure, site: *Codeview_Symbol_Inline_Site, info: Codeview_Debug_Info, allowed: [] Codeview_Line_Info_Sequence, depth: s64) -> bool {
    if !site || depth >= 64 || !cv_slice_valid(site.children.count, site.children.data) return false;

    ranges, decoded := decode_inline_site_line_info(info, procedure, site.*);
    if !decoded return false;

    defer cv_release_sequences(ranges);

    for range: ranges {
        if depth {
            contained := false;
            for parent: allowed if cv_range_contains(parent, range) contained = true;
            if !contained return false;
        }

        if !cv_place_inline(owned, range) return false;
    }

    for child: site.children if !cv_overlay_site(owned, procedure, child, info, ranges, depth + 1) return false;

    return true;
}

cv_range_contains :: (outer: Codeview_Line_Info_Sequence, inner: Codeview_Line_Info_Sequence) -> bool {
    return outer.segment_index == inner.segment_index && inner.text_offset >= outer.text_offset &&
        cast(u64) inner.text_offset + inner.length_in_bytes <= cast(u64) outer.text_offset + outer.length_in_bytes;
}

cv_place_inline :: (owned: *[..] Codeview_Line_Info_Sequence, range: Codeview_Line_Info_Sequence) -> bool {
    found: s64 = -1;

    for sequence, index: owned.* {
        if !cv_range_contains(sequence, range) continue;
        if found >= 0 return false;
        found = index;
    }

    if found < 0 return false;

    original := owned.*[found];
    replacement: [..] Codeview_Line_Info_Sequence;
    keep := false;

    defer if !keep cv_release_sequences(replacement);
    head := range.text_offset - original.text_offset;
    if head {
        clone, ok := cv_clone_sequence(original, original.text_offset, head);
        if !ok return false;
        array_add(*replacement, clone);
    }

    middle, middle_ok := cv_clone_sequence(range, range.text_offset, range.length_in_bytes);
    if !middle_ok return false;

    array_add(*replacement, middle);
    end := cast(u64) range.text_offset + range.length_in_bytes;
    original_end := cast(u64) original.text_offset + original.length_in_bytes;

    if end < original_end {
        clone, ok := cv_clone_sequence(original, cast(u32) end, cast(u32) (original_end - end));
        if !ok return false;
        array_add(*replacement, clone);
    }

    combined: [..] Codeview_Line_Info_Sequence;

    for sequence, index: owned.* {
        if index == found {
            for value: replacement array_add(*combined, value);
        } else array_add(*combined, sequence);
    }

    cv_release_blocks(*original.file_blocks);
    array_free(owned.*);
    owned.* = combined;
    array_free(replacement);
    keep = true;

    return true;
}

#scope_file
#import "Basic";
