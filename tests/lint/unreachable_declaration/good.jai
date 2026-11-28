#import "Basic";

LIMIT :: 64;

Counter :: struct {
    value: int;
}

Base :: struct {
    id: int;
}

Derived :: struct {
    using base: Base;
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

show :: (n: int) {
    print("%\n", n);
}

show :: (f: float) {
    print("%\n", f);
}

// Only compile-time code calls these.
seed :: () -> int {
    return 7;
}

SEEDED :: #run seed();

#run setup();

setup :: () {}

// Read by name, or by something outside the program.
build :: () {}

Tagged :: struct {
    id: int;
} @Reflect

on_exit :: () #c_call {}

main :: () {
    c: Counter;

    bump(*c);
    d: Derived;
    d.id = SEEDED;
    show(1);
    show(2.0);
    print("%\n", pick(.SLOW) + c.value + d.id);
}
