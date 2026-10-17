#import "Basic";

main :: () {
    text := sprint("%", formatInt(7, base = 65));
    defer free(text);
}
