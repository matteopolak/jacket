#import "Basic";

generic :: (value: $T) -> T {
    return value;
} @interception_probe
ordinary :: () -> int {
    p := cast(*u8) alloc(8);
    p[0] = 42;
    assert(p[0] == 42);
    free(p);

    return generic(3) + generic(4);
}

main :: () {
    assert(ordinary() == 7);
}
