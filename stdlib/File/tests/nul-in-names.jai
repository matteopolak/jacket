#import "Basic";
F :: #import "File";

// A name with a NUL byte in it is refused, not cut short at the NUL and applied to the shorter name.
main :: () {
    base := "nul-in-names-target";
    padded := "nul-in-names-target\0suffix";

    assert(!F.write_entire_file(padded, "x"));
    _, exists := F.file_open(base, log_errors = false);

    assert(!exists);

    assert(F.write_entire_file(base, "keep"));
    defer assert(F.file_delete(base));

    _, opened := F.file_open(padded, log_errors = false);

    assert(!opened);
    data, read_ok := F.read_entire_file(padded, log_errors = false);

    assert(!read_ok && data.count == 0);
    assert(!F.file_delete(padded));
    assert(!F.file_move(padded, "nul-in-names-moved"));
    assert(!F.file_move(base, "nul-in-names-moved\0suffix"));
    assert(!F.file_link("nul-in-names-link\0suffix", base));
    assert(!F.make_directory_if_it_does_not_exist("nul-in-names-dir\0suffix"));
    _, resolved := F.get_absolute_path(padded);

    assert(!resolved);
    assert(!F.delete_directory(padded));

    kept, kept_ok := F.read_entire_file(base);

    assert(kept_ok && kept == "keep");
    free(kept);
    print("ok\n");
}
