// Text_File_Handler: version header, comments, blank lines, line numbers and line endings.
Handler :: #import "Text_File_Handler";

main :: () -> s64 {
    {
        text := "[7]\r\n# a full-line comment\n\nalpha 1   # trailing comment\r\n   beta two words  \n#\n\rgamma";
        h: Handler.Text_File_Handler;
        Handler.start_from_memory(*h, "short", "full/path.txt", text);
        if h.failed || h.version != 7 return 1;
        line, found := Handler.consume_next_line(*h);
        if !found || line != "alpha 1" || h.line_number != 4 return 2;
        left, right := Handler.break_by_spaces(line);
        if left != "alpha" || right != "1" return 3;
        line, found = Handler.consume_next_line(*h);
        if !found || line != "beta two words" || h.line_number != 5 return 4;
        // A lone "\r" ends a line too; "#" alone strips to blank and is skipped.
        line, found = Handler.consume_next_line(*h);
        if !found || line != "gamma" || h.line_number != 8 return 5;
        line, found = Handler.consume_next_line(*h);
        if found || line != "" return 6;
        if h.short_name != "short" || h.full_path != "full/path.txt" return 7;
    }
    {
        // Without end-of-line stripping, lines that start with the comment character are dropped
        // and the rest are kept whole; blank lines survive when skipping is off.
        h: Handler.Text_File_Handler;
        h.do_version_number = false;
        h.strip_comments_from_ends_of_lines = false;
        h.auto_skip_blank_lines = false;
        h.comment_character = #char ";";
        Handler.start_from_memory(*h, "x", "x", "; note\nkey = value ; kept\n\nend");
        if h.failed || h.version != -1 return 10;
        expected :: string.["key = value ; kept", "", "end"];
        for expected {
            line, found := Handler.consume_next_line(*h);
            if !found || line != it return 11 + it_index;
        }
    }
    {
        // A missing or malformed version header marks the handler failed, and that sticks.
        h: Handler.Text_File_Handler;
        Handler.start_from_memory(*h, "a", "a", "[x]\nrest");
        if !h.failed || h.version != -1 return 20;
        Handler.start_from_memory(*h, "b", "b", "[3]\n");
        if !h.failed || h.version != 3 return 21;
        fresh: Handler.Text_File_Handler;
        Handler.start_from_memory(*fresh, "c", "c", "# only comments\n\n");
        if !fresh.failed return 22;
        fresh = .{};
        Handler.start_from_memory(*fresh, "d", "d", "  [12]  # spaced\nx");
        if fresh.failed || fresh.version != 12 return 23;
    }
    {
        ok, lines := Handler.file_to_array("mem", use_data_from_memory = true, data_from_memory = "one # c\n\n  two  \n");
        if !ok || lines.count != 2 || lines[0] != "one" || lines[1] != "two" return 30;
    }
    return 0;
}
