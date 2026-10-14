#import "Basic";

#assert OS == .WINDOWS, "this regression requires a Windows host";

U :: #import "File_Utilities";

main :: () {
    directory, exists := U.is_directory(".");

    assert(exists && directory && U.file_exists("."));
    directory, exists = U.is_directory("foundation-windows-missing/no-such-entry");
    assert(!exists && !directory);
    path := get_working_directory();

    defer free(path);
    assert(path.count > 0 && set_working_directory(path));
    time, size, found := U.file_modtime_and_size("foundation-windows-missing/no-such-entry");

    assert(!found && size == 0);
    assert(time.low == APOLLO_TIME_INVALID.low && time.high == APOLLO_TIME_INVALID.high);
    assert(!U.copy_file("foundation-windows-missing/no-such-entry", "foundation-windows-copy", silent = true));
    count: int;
    visitor :: (info: *U.File_Visit_Info, count: *int) {
        assert(!info.had_error && info.short_name.count > 0);
        count.* += 1;
    }

    assert(U.visit_files(".", false, *count, visitor, visit_directories = true));
    assert(count > 0);
    list := U.file_list(".");
    for name: list free(name);

    array_free(list);
    assert(!U.copy_directory("foundation-windows-missing/no-such-entry", "foundation-windows-copy"));
    print("ok\n");
    exit(0);
}
