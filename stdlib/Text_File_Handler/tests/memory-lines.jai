// Lines read from in-memory text: version header, full-line and trailing comments, blank lines, and no final newline.
#import "Basic";
H :: #import "Text_File_Handler";

main :: () {
    handler: H.Text_File_Handler;

    H.start_from_memory(*handler, "settings", "memory/settings", "[2]\n   # indented comment\n   \n  width   640   # pixels\nheight 480\n# tail comment\n\nname last");
    print("version=% failed=% first_line=%\n", handler.version, handler.failed, handler.line_number);
    assert(handler.version == 2 && !handler.failed);

    count := 0;

    while true {
        line, found := H.consume_next_line(*handler);
        if !found break;

        left, right := H.break_by_spaces(line);

        print("line % (%): [%] left=[%] right=[%]\n", count, handler.line_number, line, left, right);
        count += 1;
    }

    assert(count == 3);
    assert(handler.line_number == 8);
    _, found := H.consume_next_line(*handler);

    assert(!found);
    H.deinit(*handler);

    // Without a version header, no line is taken as the version.
    handler = .{};
    handler.do_version_number = false;
    H.start_from_memory(*handler, "plain", "memory/plain", "[not a version]\nnext");
    line, plain_found := H.consume_next_line(*handler);

    print("plain version=% first=[%]\n", handler.version, line);
    assert(handler.version == -1 && plain_found && line == "[not a version]");
    H.deinit(*handler);

    // file_to_array with a version header: the header is not returned, comments and blank lines are dropped.
    ok, lines := H.file_to_array("memory", use_data_from_memory = true, data_from_memory = "[5]\na\n# c\n\n  b  # x\nc", do_version_number = true);

    print("array ok=% count=%\n", ok, lines.count);
    for lines print("array [%]\n", it);

    assert(ok && lines.count == 3 && lines[0] == "a" && lines[1] == "b" && lines[2] == "c");
    for owned: lines free(owned);

    array_free(lines);

    // file_to_table with the same text keeps the same keys.
    ok_table, table := H.file_to_table("memory", use_data_from_memory = true, data_from_memory = "x\n\ny\nx\n", do_version_number = false);

    print("table ok=% count=%\n", ok_table, table.count);
    assert(ok_table && table.count == 2);
    for table free(it_index);

    print("ok\n");
}
