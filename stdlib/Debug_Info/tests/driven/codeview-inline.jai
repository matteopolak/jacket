#import "Basic";
D :: #import "Debug_Info";
#load "codeview-test-support.jai";

#run {
    assert(offset_of(D.Codeview_Symbol_Procedure, "base") == 0);
    assert(offset_of(D.Codeview_Symbol_Inline_Site, "base") == 0);
    assert(offset_of(D.Codeview_Symbol_Procedure, "parent") >= size_of(D.Codeview_Symbol));
    assert(offset_of(D.Codeview_Symbol_Procedure, "parent") % align_of(u32) == 0);
    assert(offset_of(D.Codeview_Symbol_Procedure, "name") % align_of(string) == 0);
    assert(offset_of(D.Codeview_Symbol_Inline_Site, "binary_annotations") % align_of([..] D.Codeview_Binary_Annotation) == 0);
    procedure: D.Codeview_Symbol_Procedure;

    for kind: D.Codeview_Symbol_Type.[.LPROC32, .GPROC32, .LPROC32_ID, .GPROC32_ID] {
        procedure.symbol_type = kind;
        assert(D.is_procedure(*procedure.base));
    }

    for kind: D.Codeview_Symbol_Type.[.LPROC32_ST, .GPROC32_ST, .LPROC32_DPC, .LOCAL] {
        procedure.symbol_type = kind;
        assert(!D.is_procedure(*procedure.base));
    }
}

main :: () {
    push_context {
        context.allocator = .{ count_allocation, null };
        context.logger = capture_error;
        check_all();
    }

    assert(live_allocations == 0);
    print("ok\n");
}

check_all :: () {
    check_annotation_states();
    procedure: D.Codeview_Symbol_Procedure;
    procedure.base.symbol_type = .GPROC32;
    procedure.offset = 100;
    procedure.length = 30;
    procedure.segment_index = 2;
    site: D.Codeview_Symbol_Inline_Site;
    site.inlinee = cast(D.Item_Id) 0x1000;
    info: D.Codeview_Debug_Info;

    array_add(*info.inlinee_line_info, .{ inlinee = cast(D.Type_Id) 0x1000, file_id = cast(D.Codeview_File_Checksum_Id) 7, source_line = 10 });
    defer array_free(info.inlinee_line_info);
    annotation: D.Codeview_Binary_Annotation;
    annotation.opcode = .CODE_OFFSET;
    annotation.operand1.unsigned = 5;
    array_add(*site.binary_annotations, annotation);
    annotation.opcode = .CHANGE_CODE_LENGTH;
    annotation.operand1.unsigned = 4;
    array_add(*site.binary_annotations, annotation);
    defer array_free(site.binary_annotations);
    rows, ok := D.decode_inline_site_line_info(info, procedure, site);

    assert(ok && rows.count == 1);
    defer release_sequences(rows);
    assert(rows[0].text_offset == 105 && rows[0].length_in_bytes == 4 && rows[0].segment_index == 2);
    assert(rows[0].file_blocks[0].file_id == cast(D.Codeview_File_Checksum_Id) 7);
    assert(D.get_line_number(rows[0].file_blocks[0].lines[0]) == 10);
    original: D.Codeview_Line_Info_Sequence;
    original.text_offset = 100;
    original.segment_index = 2;
    original.length_in_bytes = 30;
    line := D.Codeview_Line_Info.[.{ 0, 20 }];

    array_add(*original.file_blocks, .{ file_id = cast(D.Codeview_File_Checksum_Id) 9, lines = line });
    defer array_free(original.file_blocks);
    sequences := D.Codeview_Line_Info_Sequence.[original];
    placed, overlay := D.overlay_inlinesite(sequences, procedure, *site, info);

    assert(placed && overlay.count == 3);
    defer release_sequences(overlay);
    assert(overlay[0].text_offset == 100 && overlay[0].length_in_bytes == 5);
    assert(overlay[1].text_offset == 105 && overlay[1].length_in_bytes == 4 && overlay[1].file_blocks[0].columns.count == 1);
    assert(overlay[2].text_offset == 109 && overlay[2].length_in_bytes == 21);
    assert(overlay[2].file_blocks[0].lines[0].text_offset == 0);
    site.binary_annotations[0].operand1.unsigned = 31;
    before := live_allocations;
    failed, failed_ok := D.decode_inline_site_line_info(info, procedure, site);

    assert(!failed_ok && failed.count == 0 && live_allocations == before);
    failed_placement, failed_overlay := D.overlay_inlinesite(sequences, procedure, *site, info);

    assert(!failed_placement && failed_overlay.count == 0 && live_allocations == before);
    site.binary_annotations[0].operand1.unsigned = 5;

    for count: s64.[0, 1] {
        view := site;
        view.binary_annotations.count = count;
        failed, failed_ok = D.decode_inline_site_line_info(info, procedure, view);
        assert(!failed_ok && failed.count == 0 && live_allocations == before);
    }

    for size: s64.[8, 2] {
        deny_size = size;
        denied = false;
        failed, failed_ok = D.decode_inline_site_line_info(info, procedure, site);
        assert(denied && !failed_ok && failed.count == 0 && live_allocations == before);
    }

    child: D.Codeview_Symbol_Inline_Site;
    child.inlinee = cast(D.Item_Id) 0x1001;
    array_add(*info.inlinee_line_info, .{ inlinee = cast(D.Type_Id) 0x1001, file_id = cast(D.Codeview_File_Checksum_Id) 8, source_line = 30 });
    array_add(*child.binary_annotations, .{ opcode = .CODE_OFFSET, operand1 = .{ unsigned = 7 } });
    array_add(*child.binary_annotations, .{ opcode = .CHANGE_CODE_LENGTH, operand1 = .{ unsigned = 1 } });
    defer array_free(child.binary_annotations);
    array_add(*site.children, *child);
    defer array_free(site.children);
    nested_ok, nested := D.overlay_inlinesite(sequences, procedure, *site, info);

    assert(nested_ok && nested.count == 5);
    defer release_sequences(nested);
    assert(nested[2].text_offset == 107 && nested[2].length_in_bytes == 1 && nested[2].file_blocks[0].file_id == cast(D.Codeview_File_Checksum_Id) 8);
    child.binary_annotations[0].operand1.unsigned = 16;
    before = live_allocations;
    failed_placement, failed_overlay = D.overlay_inlinesite(sequences, procedure, *site, info);
    assert(!failed_placement && failed_overlay.count == 0 && live_allocations == before);
    assert(sequences[0].text_offset == 100 && sequences[0].length_in_bytes == 30 && line[0].line_and_delta_and_flag == 20);
}

check_annotation_states :: () {
    procedure: D.Codeview_Symbol_Procedure = .{ offset = 100, length = 30, segment_index = 3 };
    site: D.Codeview_Symbol_Inline_Site;
    info: D.Codeview_Debug_Info;

    array_add(*info.inlinee_line_info, .{ inlinee = cast(D.Type_Id) 0, file_id = cast(D.Codeview_File_Checksum_Id) 7, source_line = 10 });
    defer array_free(info.inlinee_line_info);
    array_add(*site.binary_annotations, .{ opcode = .CHANGE_LINE_OFFSET, operand1 = .{ signed = -2 } });
    array_add(*site.binary_annotations, .{ opcode = .CHANGE_FILE, operand1 = .{ unsigned = 9 } });
    array_add(*site.binary_annotations, .{ opcode = .CHANGE_LINE_END_DELTA, operand1 = .{ unsigned = 3 } });
    array_add(*site.binary_annotations, .{ opcode = .CHANGE_RANGE_KIND, operand1 = .{ unsigned = 0 } });
    array_add(*site.binary_annotations, .{ opcode = .CHANGE_COLUMN_START, operand1 = .{ unsigned = 7 } });
    array_add(*site.binary_annotations, .{ opcode = .CHANGE_COLUMN_END, operand1 = .{ unsigned = 20 } });
    array_add(*site.binary_annotations, .{ opcode = .CHANGE_COLUMN_END_DELTA, operand1 = .{ signed = -3 } });
    array_add(*site.binary_annotations, .{ opcode = .CHANGE_CODE_OFFSET_BASE, operand1 = .{ unsigned = 2 } });
    array_add(*site.binary_annotations, .{ opcode = .CODE_OFFSET, operand1 = .{ unsigned = 3 } });
    array_add(*site.binary_annotations, .{ opcode = .CHANGE_CODE_OFFSET_AND_LINE_OFFSET, operand1 = .{ unsigned = 4 }, operand2 = .{ signed = 2 } });
    array_add(*site.binary_annotations, .{ opcode = .CHANGE_CODE_LENGTH_AND_CODE_OFFSET, operand1 = .{ unsigned = 2 }, operand2 = .{ unsigned = 3 } });
    defer array_free(site.binary_annotations);
    ranges, ok := D.decode_inline_site_line_info(info, procedure, site);

    assert(ok && ranges.count == 1 && ranges[0].text_offset == 105 && ranges[0].length_in_bytes == 9);
    defer release_sequences(ranges);
    block := ranges[0].file_blocks[0];

    assert(block.file_id == cast(D.Codeview_File_Checksum_Id) 9 && block.lines.count == 3 && block.columns.count == 3);
    assert(block.lines[1].text_offset == 4 && block.lines[2].text_offset == 7 && block.columns[2] == 7);
    assert(D.get_line_number(block.lines[0]) == 8 && D.get_line_number(block.lines[1]) == 10);
    assert(!D.line_info_is_for_statement(block.lines[0]) && D.get_line_number_end_delta(block.lines[0]) == 3);
    before := live_allocations;
    missing: D.Codeview_Debug_Info;
    failed, failed_ok := D.decode_inline_site_line_info(missing, procedure, site);

    assert(!failed_ok && failed.count == 0 && live_allocations == before);
    site.binary_annotations[0].operand1.signed = -11;
    failed, failed_ok = D.decode_inline_site_line_info(info, procedure, site);
    assert(!failed_ok && failed.count == 0 && live_allocations == before);
}
