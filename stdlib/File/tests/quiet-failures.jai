#import "Basic";
F :: #import "File";

// A missing file is an ordinary answer: the result says so and nothing is printed.
main :: () {
    name := "quiet-failures-absent.txt";

    text, ok := F.read_entire_file(name);

    assert(!ok && text.count == 0);

    _, opened := F.file_open(name);

    assert(!opened);

    info, mapped := F.map_entire_file_start(name);

    assert(!mapped && info.data.count == 0);

    // Asking for the log still works: this is the one line on standard error.
    _, ok = F.read_entire_file(name, log_errors = true);
    assert(!ok);

    print("ok\n");
}
