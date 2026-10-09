// Reads standard input a line at a time with `read_stdin_line()`: the crates/jaic-cli
// native test pipes in "one\r\n\ntwo words\nlast" and expects every line back, the last one
// included though it has no newline.
#import "Basic";
#import "File";

main :: () {
    while true {
        line, ok := read_stdin_line();
        if !ok break;
        print("[%]", line);
    }
    print("\n");
}
