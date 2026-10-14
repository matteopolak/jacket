#import "Basic";
F :: #import "File";
U :: #import "File_Utilities";

main :: () {
    root := "foundation-invalid-path-test";

    assert(F.make_directory_if_it_does_not_exist(root));
    source := "foundation-invalid-path-test/source";
    destination := "foundation-invalid-path-test/destination";

    assert(F.write_entire_file(source, "source bytes"));
    assert(F.write_entire_file(destination, "keep destination"));

    defer {
        assert(F.file_delete(source));
        assert(F.file_delete(destination));
        assert(F.delete_directory(root));
    }

    assert(!U.file_exists("foundation-invalid-path-test\0suffix"));
    directory, success := U.is_directory("foundation-invalid-path-test\0suffix");

    assert(!directory && !success);
    time, size, found := U.file_modtime_and_size("foundation-invalid-path-test/source\0suffix");

    assert(!found && size == 0 && time.low == APOLLO_TIME_INVALID.low && time.high == APOLLO_TIME_INVALID.high);
    assert(!U.copy_file(source, "foundation-invalid-path-test/destination\0suffix"));
    assert(!U.copy_file("foundation-invalid-path-test/source\0suffix", destination));
    assert(!U.copy_directory("foundation-invalid-path-test\0suffix", "foundation-invalid-path-test/copy"));
    assert(!U.atomic_file_replace_or_just_rename_if_destination_file_does_not_exist("foundation-invalid-path-test/source\0suffix", destination));
    count: int;

    assert(!U.visit_files("foundation-invalid-path-test\0suffix", false, *count, visitor));
    assert(count == 0);
    assert(!set_working_directory("foundation-invalid-path-test\0suffix"));
    contents, read_ok := F.read_entire_file(destination);

    defer free(contents);
    assert(read_ok && contents == "keep destination");
    assert(U.file_exists(source) && !U.file_exists("foundation-invalid-path-test/copy"));
    print("ok\n");
}

visitor :: (info: *U.File_Visit_Info, count: *int) {
    count.* += 1;
}
