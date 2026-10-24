#import "Basic";

#scope_file

LIMIT :: 64;

Counter :: struct {
    value: int;
}

Mode :: enum {
    SLOW;
    FAST;
}

bump :: (c: *Counter) {
    c.value += 1;
}

pick :: (m: Mode) -> int {
    if m == .FAST return LIMIT;
    return 1;
}

// Read by name: the build system, `for` and the debugger find these.
build :: () {}

Tagged :: struct {
    id: int;
} @Reflect

on_exit :: () #c_call {}

#scope_export

visible :: () {}

main :: () {
    c: Counter;
    bump(*c);
    print("%\n", pick(.SLOW) + c.value);
}
