// test: should_fail("assertion failed")
#import "Basic";

main :: () {
    text := sprint("%", formatInt(7, base = 65));
    defer free(text);
}
