#import "Basic";

main :: () {
    buffer := alloc(64);
    names := string.["a", "b", "c"];

    for names {
        defer free(buffer);
        print("%\n", it);
    }

    items: [..] int;
    i := 0;

    while i < 2 {
        i += 1;
        defer array_reset(*items);
    }
}
