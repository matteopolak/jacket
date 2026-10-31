// test: skip_on(.WASM, .WINDOWS, reason = "a directory in use cannot be deleted on Windows, and on wasm the working directory is a name that stays after the directory is gone") serial
#import "Basic";
#import "File";

main :: () {
    directory := "/tmp/jai-deleted-working-directory";

    assert(make_directory_if_it_does_not_exist(directory));
    assert(set_working_directory(directory));
    assert(delete_directory(directory));

    // The directory no longer exists, so the system cannot name it: the result is empty, not an assertion.
    current := get_working_directory();

    defer free(current);
    assert(current.count == 0);
    assert(set_working_directory("/"));
    assert(get_working_directory() == "/");
    print("ok\n");
    exit(0);
}
