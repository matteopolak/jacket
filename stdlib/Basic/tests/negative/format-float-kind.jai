#import "Basic";

main :: () {
    text := sprint("%", formatFloat(7));
    defer free(text);
}
