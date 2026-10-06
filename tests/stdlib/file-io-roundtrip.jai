// File I/O against whatever file system the host provides: the real one natively, the in-memory
// virtual file system (workspace files plus whatever the program writes) in the browser.
#import "Basic";
#import "File";
#import "File_Utilities";
#import "String";

main :: () {
    // The program's own source is readable relative to the working directory (the workspace in the browser).
    own, own_ok := read_entire_file("file-io-roundtrip.jai");
    assert(own_ok && begins_with(own, "// File I/O"), "own source");

    root :: "/tmp/jaic_file_io_roundtrip";
    delete_directory(root);
    assert(make_directory_if_it_does_not_exist(root, true));
    assert(is_directory(root));

    path := tprint("%/hello.txt", root);
    assert(!file_exists(path));
    assert(write_entire_file(path, "hello, files\n"));
    assert(file_exists(path));
    text, ok := read_entire_file(path);
    assert(ok && text == "hello, files\n", "read back '%'", text);

    // Append through file_open, seek and measure.
    file, opened := file_open(path, for_writing = true, keep_existing_content = true);
    assert(opened);
    assert(file_write(*file, "more\n"));
    // A handle opened for writing reads too (Windows: GENERIC_READ alongside GENERIC_WRITE).
    assert(file_set_position(file, 0));
    head: [5] u8;
    head_ok, head_count := file_read(file, head.data, head.count);
    assert(head_ok && head_count == 5 && cast(string) head == "hello", "read through the writing handle");
    assert(file_close(*file));
    sized, sized_ok := file_open(path);
    assert(sized_ok);
    length, length_ok := file_length_checked(sized);
    assert(file_close(*sized));
    assert(length_ok && length == 18, "length %", length);
    text, ok = read_entire_file(path);
    assert(text == "hello, files\nmore\n", "after append '%'", text);

    // Directories, listing, copy, move, delete.
    sub := tprint("%/sub", root);
    assert(make_directory_if_it_does_not_exist(sub));
    assert(write_entire_file(tprint("%/inner.txt", sub), "inner"));
    assert(copy_file(path, tprint("%/copy.txt", root)));
    assert(file_move(tprint("%/copy.txt", root), tprint("%/moved.txt", root)));
    names := file_list(root, true);
    found: [3] bool;
    for names {
        if ends_with(it, "hello.txt") found[0] = true;
        if ends_with(it, "moved.txt") found[1] = true;
        if ends_with(it, "inner.txt") found[2] = true;
    }
    assert(found[0] && found[1] && found[2], "listing % has %", names.count, found);
    assert(!file_exists(tprint("%/copy.txt", root)));

    // Missing files fail cleanly.
    _, missing_ok := read_entire_file(tprint("%/missing.txt", root), false);
    assert(!missing_ok);

    assert(delete_directory(root));
    assert(!file_exists(path));
    print("ok\n");
}
