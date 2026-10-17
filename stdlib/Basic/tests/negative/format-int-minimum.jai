#import "Basic";

main :: () {
    text := sprint("%", formatInt(7, minimum_digits = -1));
    defer free(text);
}
