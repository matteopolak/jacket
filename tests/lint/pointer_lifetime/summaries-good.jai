#import "Basic";

grow :: (a: *[..] int) {} @GrowsArray
maybe :: (a: *[..] int, flag: bool) {
    if flag grow(a);
}

Box :: struct {
    entries: [..] int;
    other: [..] int;
}

field_wrapper :: (b: *Box) {
    grow(*b.entries);
}

sink :: (p: *int) -> int {
    return p.*;
}

conditional :: (flag: bool) {
    a: [..] int;

    array_add(*a, 1);
    p := *a[0];

    maybe(*a, flag);
    sink(p);
}

other_field :: () {
    b: Box;

    array_add(*b.other, 1);
    p := *b.other[0];

    field_wrapper(*b);
    sink(p);
}

// No optimistic recursive guarantee: the base path does not grow.
cycle_a :: (a: *[..] int, n: int) {
    if n > 0 cycle_b(a, n - 1);
}

cycle_b :: (a: *[..] int, n: int) {
    if n > 0 cycle_a(a, n - 1);
    grow(a);
}

recursive_safe_exit :: () {
    a: [..] int;

    array_add(*a, 1);
    p := *a[0];

    cycle_a(*a, 0);
    sink(p);
}

rebind :: (a: *[..] int, other: [..] int) {
    a.* = other;
    grow(a);
}

rebound_storage_is_not_the_old_generation :: () {
    a: [..] int;
    other: [..] int;

    array_add(*a, 1);
    p := *a[0];

    rebind(*a, other);
    sink(p);
}

alias_write :: (a: *[..] int, b: *[..] int, flag: bool) {
    p := a;
    if flag p = b;

    other: [..] int;

    p.* = other;
    grow(a);
}

uncertain_alias_write_does_not_prove_old_generation_invalid :: (flag: bool) {
    a: [..] int;
    b: [..] int;

    array_add(*a, 1);
    p := *a[0];

    alias_write(*a, *b, flag);
    sink(p);
}

never :: (a: *[..] int) {
    grow(a);
    while true {}
}

unreachable_after_nonreturning_call :: () {
    a: [..] int;

    array_add(*a, 1);
    p := *a[0];

    never(*a);
    sink(p);
}

main :: () {}
