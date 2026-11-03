#import "Basic";

main :: () {
    buffer := alloc(64);

    defer free(buffer);

    names := string.["a", "b", "c"];

    for names {
        print("%\n", it);
    }
}
