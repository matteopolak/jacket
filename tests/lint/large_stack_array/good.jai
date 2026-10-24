#import "Basic";

// Globals do not live on the stack.
table: [1 << 20] u8;

render :: (input: [100000] u8) -> int {
    small: [4096] u8;

    exactly: [65536] u8;

    heap := NewArray(1 << 20, u8);

    defer free(heap.data);

    resizable: [..] u8;

    defer array_free(resizable);

    total := 0;

    for small total += it;
    for exactly total += it;

    return total + heap.count + resizable.count + table.count + input.count;
}

main :: () {
    print("%\n", render(.{}));
}
