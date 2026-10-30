// test: should_fail("FormatInt requires an integer value")
#import "Basic";

main :: () {
    text := sprint("%", formatInt(true));
    defer free(text);
}
