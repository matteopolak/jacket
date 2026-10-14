#import "Basic";

main :: () {
    text := sprint("%", formatInt(true));
    defer free(text);
}
