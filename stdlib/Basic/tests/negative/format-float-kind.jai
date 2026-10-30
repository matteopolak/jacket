// test: should_fail("FormatFloat requires a float value")
#import "Basic";

main :: () {
    text := sprint("%", formatFloat(7));
    defer free(text);
}
