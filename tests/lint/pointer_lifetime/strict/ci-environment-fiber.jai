#import "Basic";
#import "Extensions/Environment";
Io :: #import "Extensions/Io";

saved: string;
other: string;
bad_saved: string;

main :: () {
    saved = environment_variable("PATH");
    other = environment_variable("PATH");
    original := copy_string(saved);

    defer free(original);
    Io.io_run(() {
        mark := get_temporary_storage_mark();
        defer set_temporary_storage_mark(mark);
        reset_temporary_storage();
        assert(saved.count > 0);
    });
    assert(saved == original);
    assert(other == original);
    mark := get_temporary_storage_mark();

    bad_saved = environment_variable("PATH");
    set_temporary_storage_mark(mark);
}
