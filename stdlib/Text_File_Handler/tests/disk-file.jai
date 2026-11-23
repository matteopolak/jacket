// Files read from disk: CRLF endings, a version header, a missing optional file, and a missing required file reported through the logger.
#import "Basic";
H :: #import "Text_File_Handler";
F :: #import "File";

reports: int;
report_kinds: int;

count_reports :: (message: string, data: *void, info: Log_Info) {
    reports += 1;
    if (info.common_flags & .ERROR) != 0 report_kinds += 1;
}

main :: () {
    path := "text-handler-disk.txt";

    assert(F.write_entire_file(path, "[4]\r\n# header comment\r\n\r\nfirst 1\r\nsecond 2 # trailing\r\nthird 3"));

    handler: H.Text_File_Handler;

    H.start_file(*handler, "disk", path);
    print("failed=% version=% data_count=%\n", handler.failed, handler.version, handler.file_data.count);
    assert(!handler.failed && handler.version == 4);

    count := 0;

    while true {
        line, found := H.consume_next_line(*handler);
        if !found break;
        print("[%] %\n", handler.line_number, line);
        count += 1;
    }

    assert(count == 3);
    H.deinit(*handler);
    assert(handler.file_data.count == 0);

    // An optional missing file is quiet and gives no lines.
    H.start_file(*handler, "optional", "text-handler-missing.txt", optional = true);
    _, found := H.consume_next_line(*handler);

    print("optional failed=% found=% reports=%\n", handler.failed, found, reports);
    assert(!handler.failed && !found && reports == 0);
    H.deinit(*handler);

    // A required missing file fails and is reported once.
    previous := context.logger;

    context.logger = count_reports;
    H.start_file(*handler, "required", "text-handler-missing.txt");
    context.logger = previous;
    print("required failed=% reports=% errors=%\n", handler.failed, reports, report_kinds);
    assert(handler.failed && reports == 1 && report_kinds == 1);
    H.deinit(*handler);

    assert(F.file_delete(path));
    print("ok\n");
}
