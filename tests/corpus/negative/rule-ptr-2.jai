// rules: ptr.2
// `remove` is not allowed while iterating a fixed array: its length cannot shrink.
main :: () {
    fixed: [3] int = .[1, 2, 3];
    for fixed {
        if it == 2 remove it;
    }
}
