#import "Basic";
H :: #import "Text_File_Handler";
F :: #import "File";
T :: #import "Hash_Table";

reports: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0 && (info.common_flags & (.ERROR | .WARNING)) != 0);
    reports += 1;
}

main :: () {
    handler: H.Text_File_Handler;

    H.start_from_memory(*handler, "config", "memory/config", "[7]\r\n# comment\n\n  alpha 9 # note\r\nbeta");
    assert(!handler.failed && handler.version == 7 && handler.line_number == 1);
    line, found := H.consume_next_line(*handler);

    assert(found && line == "alpha 9" && handler.line_number == 4);
    left, right := H.break_by_spaces(line);

    assert(left == "alpha" && right == "9");
    line, found = H.consume_next_line(*handler);
    assert(found && line == "beta" && handler.line_number == 5);
    line, found = H.consume_next_line(*handler);
    assert(!found);
    H.deinit(*handler);
    previous := context.logger;

    context.logger = capture;
    defer context.logger = previous;
    H.start_from_memory(*handler, "bad", "memory/bad", "[x]\nrest");
    assert(handler.failed && reports == 1);
    H.start_from_memory(*handler, "new", "memory/new", "[3]\nline");
    assert(handler.failed && handler.version == 3);
    handler = .{};
    handler.do_version_number = false;
    H.start_from_memory(*handler, "x", "x", "one\rtwo\nthree\r\nfour");

    for expected: string.["one", "two", "three", "four"] {
        line, found = H.consume_next_line(*handler);
        assert(found && line == expected);
    }

    ok, lines := H.file_to_array("memory", use_data_from_memory = true, data_from_memory = " alpha # c\n\n beta\n", do_trim = false, skip_blank_lines = false);

    assert(ok && lines.count == 3 && lines[0] == " alpha " && lines[1] == "" && lines[2] == " beta");
    for owned: lines free(owned);

    array_free(lines);
    assert(F.write_entire_file("portable-text-handler.tmp", "beta\nalpha\nbeta\n"));
    handler = .{};
    handler.do_version_number = false;
    H.start_file(*handler, "disk", "portable-text-handler.tmp");
    assert(!handler.failed && handler.file_data_to_free.count > 0);
    line, found = H.consume_next_line(*handler);
    assert(found && line == "beta");
    H.deinit(*handler);
    assert(handler.file_data_to_free.data == null && handler.file_data.count == 0);
    H.start_file(*handler, "optional", "portable-missing-text-handler.tmp", optional = true);
    assert(!handler.failed && handler.file_data.count == 0);
    H.warn(handler, "warning %", 7);
    H.error(handler, "error %", "message");
    assert(reports == 3);
    table_ok, table := H.file_to_table("portable-text-handler.tmp");

    assert(table_ok && table.count == 2 && T.table_contains(*table, "alpha") && T.table_contains(*table, "beta"));
    for table free(it_index);

    T.deinit(*table);
    assert(F.file_delete("portable-text-handler.tmp"));
    print("ok\n");
}
