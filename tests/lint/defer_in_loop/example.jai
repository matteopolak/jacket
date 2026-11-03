#import "Basic";

main :: () {
    buffer := alloc(64);
    names := string.["a", "b", "c"];

    for names {
        defer free(buffer);
        print("%\n", it);
    }
}
