copy_and_truncate_blocks :: (sequence: *Codeview_Line_Info_Sequence, old_sequence_text_offset: u32) {
    if !sequence {
        log_error("Debug_Info: null CodeView line sequence\n");
        return;
    }

    blocks, ok := cv_truncate_blocks(sequence.*, old_sequence_text_offset);
    if !ok {
        log_error("Debug_Info: invalid CodeView line blocks\n");
        return;
    }

    // The old blocks remain owned by their source. This operation detaches a
    // caller's sliced view; it never frees storage potentially shared by it.
    sequence.file_blocks = blocks;
}

#scope_module

cv_release_blocks :: (blocks: *[..] Codeview_Lines_File_Block) {
    for block: blocks.* {
        free(block.lines.data);
        free(block.columns.data);
    }

    array_free(blocks.*);
}

cv_truncate_blocks :: (sequence: Codeview_Line_Info_Sequence, old_start: u32) -> [..] Codeview_Lines_File_Block, bool {
    result: [..] Codeview_Lines_File_Block;
    keep := false;

    defer if !keep cv_release_blocks(*result);
    if sequence.file_blocks.count < 0 || (sequence.file_blocks.count && !sequence.file_blocks.data) return .{}, false;

    start := cast(u64) sequence.text_offset;
    end := start + sequence.length_in_bytes;

    if end > 0x100000000 || start < old_start return .{}, false;

    columns := (cast(u16) sequence.flags & 1) != 0;

    for block: sequence.file_blocks {
        if block.lines.count < 0 || (block.lines.count && !block.lines.data) || block.columns.count < 0 ||
            (block.columns.count && !block.columns.data) return .{}, false;
        if block.lines.count > 0x7fffffffffffffff / 8 return .{}, false;
        if columns && block.columns.count != block.lines.count && block.columns.count != block.lines.count * 2 return .{}, false;
        if !columns && block.columns.count return .{}, false;

        for row, index: block.lines {
            absolute := cast(u64) old_start + row.text_offset;
            if absolute > 0xffffffff || (index && row.text_offset < block.lines[index - 1].text_offset) return .{}, false;
        }

    }

    for block, block_index: sequence.file_blocks {
        for previous: 0..block_index - 1 {
            for row: block.lines for other : sequence.file_blocks[previous].lines if row.text_offset == other.text_offset return .{}, false;
        }
    }

    for block, block_index: sequence.file_blocks {
        if !sequence.length_in_bytes continue;

        column_stride: s64 = 1;
        if columns && block.columns.count == block.lines.count * 2 column_stride = 2;

        count: s64;

        for row, index: block.lines {
            absolute := cast(u64) old_start + row.text_offset;
            next := cv_line_successor(sequence, block_index, index, old_start, end);
            if max(start, absolute) >= min(end, next) continue;

            count += 1;
        }

        if !count continue;

        output: Codeview_Lines_File_Block;
        output.file_id = block.file_id;
        array_add(*result, output);
        stored := *result[result.count - 1];
        stored.lines = .{ count, cast(*Codeview_Line_Info) alloc(count * size_of(Codeview_Line_Info)) };
        if !stored.lines.data return .{}, false;

        if columns {
            stored.columns = .{ count * column_stride, cast(*u16) alloc(count * column_stride * size_of(u16)) };
            if !stored.columns.data return .{}, false;
        }

        destination: s64;

        for row, index: block.lines {
            absolute := cast(u64) old_start + row.text_offset;
            next := cv_line_successor(sequence, block_index, index, old_start, end);

            if max(start, absolute) >= min(end, next) continue;

            row.text_offset = cast(u32) (max(start, absolute) - start);
            stored.lines[destination] = row;

            if columns {
                for component: 0..column_stride - 1 stored.columns[destination * column_stride + component] = block.columns[index * column_stride + component];
            }

            destination += 1;
        }
    }

    keep = true;

    return result, true;
}

cv_line_successor :: (sequence: Codeview_Line_Info_Sequence, block_index: s64, row_index: s64, old_start: u32, end: u64) -> u64 {
    block := sequence.file_blocks[block_index];
    offset := block.lines[row_index].text_offset;

    if row_index + 1 < block.lines.count && block.lines[row_index + 1].text_offset == offset return cast(u64) old_start + offset;

    next := end;

    for other: sequence.file_blocks for row : other.lines {
        if row.text_offset > offset next = min(next, cast(u64) old_start + row.text_offset);
    }

    return next;
}

#scope_file
#import "Basic";
