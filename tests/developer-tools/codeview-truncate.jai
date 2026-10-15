#import "Basic";
D :: #import "Debug_Info";

main :: () {
    check_repeated();
    check_interleaved();
    rows := D.Codeview_Line_Info.[.{ 0, 10 }, .{ 10, 20 }, .{ 20, 30 }];
    columns := u16.[1, 2, 3, 4, 5, 6];
    old: D.Codeview_Line_Info_Sequence;
    old.text_offset = 100;
    old.length_in_bytes = 30;
    old.flags = .HAS_COLUMNS;
    array_add(*old.file_blocks, .{ file_id = cast(D.Codeview_File_Checksum_Id) 7, lines = rows, columns = columns });
    defer array_free(old.file_blocks);
    clipped := old;
    clipped.text_offset = 115;
    clipped.length_in_bytes = 9;
    D.copy_and_truncate_blocks(*clipped, old.text_offset);

    defer {
        for block: clipped.file_blocks {
            free(block.lines.data);
            free(block.columns.data);
        }

        array_free(clipped.file_blocks);
    }

    assert(clipped.file_blocks.data != old.file_blocks.data && clipped.file_blocks.count == 1);
    block := clipped.file_blocks[0];

    assert(block.lines.count == 2 && block.lines[0].text_offset == 0 && block.lines[0].line_and_delta_and_flag == 20);
    assert(block.lines[1].text_offset == 5 && block.lines[1].line_and_delta_and_flag == 30);
    assert(block.columns.count == 4 && block.columns[0] == 3 && block.columns[3] == 6);
    block.lines[0].line_and_delta_and_flag = 99;
    block.columns[0] = 99;
    assert(rows[1].line_and_delta_and_flag == 20 && columns[2] == 3);
    print("ok\n");
}

check_interleaved :: () {
    first := D.Codeview_Line_Info.[.{ 0, 1 }, .{ 20, 3 }];
    second := D.Codeview_Line_Info.[.{ 10, 2 }];
    source: D.Codeview_Line_Info_Sequence;

    array_add(*source.file_blocks, .{ file_id = cast(D.Codeview_File_Checksum_Id) 1, lines = first });
    array_add(*source.file_blocks, .{ file_id = cast(D.Codeview_File_Checksum_Id) 2, lines = second });
    defer array_free(source.file_blocks);
    clipped := source;
    clipped.text_offset = 115;
    clipped.length_in_bytes = 4;
    D.copy_and_truncate_blocks(*clipped, 100);

    defer {
        for block: clipped.file_blocks free(block.lines.data);
        array_free(clipped.file_blocks);
    }

    assert(clipped.file_blocks.count == 1 && clipped.file_blocks[0].file_id == cast(D.Codeview_File_Checksum_Id) 2);
    assert(clipped.file_blocks[0].lines.count == 1 && clipped.file_blocks[0].lines[0].text_offset == 0);
}

check_repeated :: () {
    rows := D.Codeview_Line_Info.[.{ 0, 1 }, .{ 10, 2 }, .{ 10, 3 }, .{ 20, 4 }];
    original: D.Codeview_Line_Info_Sequence;
    original.length_in_bytes = 30;
    array_add(*original.file_blocks, .{ lines = rows });
    defer array_free(original.file_blocks);
    clipped := original;

    D.copy_and_truncate_blocks(*clipped, 0);

    defer {
        for block: clipped.file_blocks free(block.lines.data);
        array_free(clipped.file_blocks);
    }

    assert(clipped.file_blocks[0].lines.count == 3);
    assert(clipped.file_blocks[0].lines[1].line_and_delta_and_flag == 3 && clipped.file_blocks[0].lines[2].line_and_delta_and_flag == 4);
}
