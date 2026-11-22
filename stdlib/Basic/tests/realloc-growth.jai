#import "Basic";

// A block grows through a cache line, and an array of 8-byte floats grows with it: the C
// library gets a `size_t`-wide size on every target, so contents and sizes survive.
main :: () {
    memory := cast(*u8) alloc(16);
    for i: 0..15 memory[i] = cast,trunc(u8) i;

    for size: cast([4] s64) .[64, 256, 4096, 24] {
        memory = cast(*u8) realloc(memory, size, 16);
        assert(memory != null);
        for i: 0..15 assert(memory[i] == cast(u8) i);
    }

    free(memory);

    values: [..] float64;
    for i: 0..99 array_add(*values, cast(float64) i * 0.5);

    assert(values.count == 100 && values[99] == 49.5);
    array_free(values);
}
