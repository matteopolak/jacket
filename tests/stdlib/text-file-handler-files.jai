// Text_File_Handler reading real files: start_file with and without a version header, optional
// and required missing files, error and warn reports, file_to_array and file_to_table.
#import "Basic";
#import "File";
#import "String";
#import "Text_File_Handler";
Hash_Table :: #import "Hash_Table";

ROOT :: ".scratch/text-file-handler-files";

path :: (name: string) -> string {
    return tprint("%/%", ROOT, name);
}

// Counts the reports error and warn send to the logger.
reports: [..] string;

capture_logger :: (message: string, data: *void, info: Log_Info) {
    tag := ifx info.common_flags & .ERROR then "error: " else ifx info.common_flags & .WARNING then "warning: " else "";
    array_add(*reports, sprint("%1%2", tag, message));
}

main :: () {
    delete_directory(ROOT);
    assert(make_directory_if_it_does_not_exist(ROOT, recursive = true));
    config := "[3]  # version three\r\nname    jaic  \n\n# comment\nlimit 10 # trailing\r\nünïcode  välue\n";
    assert(write_entire_file(path("config.txt"), config));

    {
        handler: Text_File_Handler;
        start_file(*handler, "config", path("config.txt"));
        assert(!handler.failed && handler.version == 3, "version %", handler.version);
        assert(handler.short_name == "config" && handler.full_path == path("config.txt"));
        line, found := consume_next_line(*handler);
        assert(found && line == "name    jaic" && handler.line_number == 2, "line % '%'", handler.line_number, line);
        left, right := break_by_spaces(line);
        assert(left == "name" && right == "jaic");
        line, found = consume_next_line(*handler);
        assert(found && line == "limit 10" && handler.line_number == 5);
        line, found = consume_next_line(*handler);
        assert(found && line == "ünïcode  välue");
        left, right = break_by_spaces(line);
        assert(left == "ünïcode" && right == "välue");
        line, found = consume_next_line(*handler);
        assert(!found);

        // error and warn name the file and line; capture them through the logger.
        old_logger := context.logger;
        context.logger = capture_logger;
        error(handler, "bad value %", 42);
        warn(handler, "odd value %", "x");
        context.logger = old_logger;
        assert(reports.count == 2, "% reports", reports.count);
        assert(contains(reports[0], "bad value 42") && contains(reports[0], "config.txt") && contains(reports[0], "6"), "error report '%'", reports[0]);
        assert(contains(reports[1], "odd value x"), "warning report '%'", reports[1]);
        assert(begins_with(reports[0], "error") && begins_with(reports[1], "warning"), "report kinds '%' '%'", reports[0], reports[1]);
        deinit(*handler);
        assert(handler.file_data_to_free.count == 0);
    }
    {
        // A required file that is missing fails; an optional one does not.
        handler: Text_File_Handler;
        old_logger := context.logger;
        context.logger = capture_logger;
        start_file(*handler, "missing", path("missing.txt"));
        context.logger = old_logger;
        assert(handler.failed && handler.version == -1);
        _, found := consume_next_line(*handler);
        assert(!found);

        optional: Text_File_Handler;
        start_file(*optional, "missing", path("missing.txt"), optional = true);
        assert(!optional.failed && optional.full_path == path("missing.txt"));
        _, found = consume_next_line(*optional);
        assert(!found);

        // Reusing a handler for another file releases the first buffer.
        reused: Text_File_Handler;
        reused.do_version_number = false;
        start_file(*reused, "config", path("config.txt"));
        first, first_found := consume_next_line(*reused);
        assert(first_found && first == "[3]", "without a version header the first line is '%'", first);
        assert(write_entire_file(path("second.txt"), "second file\n"));
        start_file(*reused, "second", path("second.txt"));
        line, line_found := consume_next_line(*reused);
        assert(line_found && line == "second file" && reused.line_number == 1);
        deinit(*reused);
    }
    {
        // An empty file with a required version header fails.
        assert(write_entire_file(path("empty.txt"), ""));
        handler: Text_File_Handler;
        start_file(*handler, "empty", path("empty.txt"));
        assert(handler.failed);
        deinit(*handler);
    }
    {
        list := "beta\nalpha # first\n\n  beta  \ngamma\n";
        assert(write_entire_file(path("list.txt"), list));
        ok, lines := file_to_array(path("list.txt"));
        assert(ok && lines.count == 4, "% lines", lines.count);
        assert(lines[0] == "beta" && lines[1] == "alpha" && lines[2] == "beta" && lines[3] == "gamma");

        ok, lines = file_to_array(path("list.txt"), do_trim = false, skip_blank_lines = false);
        assert(ok && lines.count == 5 && lines[1] == "alpha " && lines[2] == "" && lines[3] == "  beta  ", "untrimmed % lines", lines.count);

        ok, lines = file_to_array(path("missing.txt"));
        assert(!ok && lines.count == 0);

        table_ok, table := file_to_table(path("list.txt"));
        assert(table_ok && table.count == 3, "table of % entries", table.count);
        for string.["alpha", "beta", "gamma"] assert(Hash_Table.table_contains(*table, it), "table has %", it);
        assert(!Hash_Table.table_contains(*table, ""));

        // A version header can be required here too.
        assert(write_entire_file(path("versioned.txt"), "[1]\nx\n"));
        ok, lines = file_to_array(path("versioned.txt"), do_version_number = true);
        assert(ok && lines.count == 1 && lines[0] == "x", "versioned: % %", ok, lines.count);
        ok, lines = file_to_array(path("list.txt"), do_version_number = true);
        assert(!ok, "a file without its version header");
    }

    assert(delete_directory(ROOT));
    print("ok\n");
}
