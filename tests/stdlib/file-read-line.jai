// file_read_line: lines of a file without their newline (or the carriage return before it), empty
// lines, a last line with no newline, and the end of input.
#import "Basic";
#import "File";

main :: () {
    path := "/tmp/jai-file-read-line.txt";
    assert(write_entire_file(path, "one\r\n\ntwo words\nlast"));
    defer file_delete(path);

    input, opened := file_open(path);
    assert(opened);
    defer file_close(*input);

    expected := string.["one", "", "two words", "last"];
    for expected {
        line, ok := file_read_line(input);
        assert(ok);
        assert(line == it);
        free(line);
    }
    line, ok := file_read_line(input);
    assert(!ok);
    assert(line.count == 0);

    // A file that ends with a newline has no extra empty line.
    assert(write_entire_file(path, "x\n"));
    again, reopened := file_open(path);
    assert(reopened);
    defer file_close(*again);
    first, first_ok := file_read_line(again);
    assert(first_ok && first == "x");
    free(first);
    _, second_ok := file_read_line(again);
    assert(!second_ok);

    print("ok\n");
}
