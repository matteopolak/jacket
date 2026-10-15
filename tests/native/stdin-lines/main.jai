// Native pipe/EOF regression through reviewed File.file_read. The test owns line
// splitting; File has no public read_stdin_line helper. Empty lines, CRLF and the
// final unterminated line remain checked by the Rust caller.
#import "Basic";
#import "File";

main :: () {
    input: File;

    #if OS == .WINDOWS input.handle = W.GetStdHandle(W.STD_INPUT_HANDLE);
    else input.handle = P.fdopen(P.STDIN_FILENO, "rb");

    assert(is_valid(input));
    defer file_close(*input);
    bytes: [..] u8;

    defer array_free(bytes);

    while true {
        byte: u8;
        ok, count := file_read(input, *byte, 1);

        assert(ok && count >= 0 && count <= 1);

        if count == 0 {
            if bytes.count > 0 print("[%]", string.{ bytes.count, bytes.data });
            break;
        }

        if byte == #char "\n" {
            length := bytes.count;
            if length > 0 && bytes[length - 1] == #char "\r" length -= 1;

            print("[%]", string.{ length, bytes.data });
            array_reset(*bytes);
        } else array_add(*bytes, byte);
    }

    print("\n");
}

#scope_file
#if OS == .WINDOWS W :: #import "Windows";
else P :: #import "POSIX";
