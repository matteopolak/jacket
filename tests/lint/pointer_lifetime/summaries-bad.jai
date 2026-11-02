#import "Basic";

grow :: (a: *[..] int) {} @GrowsArray
wrapper :: (a: *[..] int) {
    grow(a);
}

push :: (a: *[..] int) {
    wrapper(a);
}

sink :: (p: *int) -> int {
    return p.*;
}

chain :: () {
    a: [..] int;

    array_add(*a, 1);
    p := *a[0];

    push(*a);
    sink(p);
}

Box :: struct {
    entries: [..] int;
    other: [..] int;
}

field_wrapper :: (b: *Box) {
    grow(*b.entries);
}

field_effect :: () {
    b: Box;

    array_add(*b.entries, 1);
    p := *b.entries[0];

    field_wrapper(*b);
    sink(p);
}

Storage :: struct {
    memory: *int;
}

resize :: (s: *Storage, allocator: Allocator) {
    s.memory = cast(*int) allocator.proc(.RESIZE, 16, 8, s.memory, allocator.data);
}

primitive_wrapper :: (s: *Storage, allocator: Allocator) {
    resize(s, allocator);
}

allocator_effect :: () {
    s: Storage;
    s.memory = New(int);
    p := s.memory;

    primitive_wrapper(*s, context.allocator);
    sink(p);
}

lookup :: (b: *Box) -> *int {
    return b.entries.data;
}

returned_projection :: () {
    b: Box;

    array_add(*b.entries, 1);
    p := lookup(*b);

    grow(*b.entries);
    sink(p);
}

// Recursive groups gain only effects that every returning path establishes.
left :: (a: *[..] int, n: int) {
    if n > 0 right(a, n - 1);
    grow(a);
}

right :: (a: *[..] int, n: int) {
    if n > 0 left(a, n - 1);
    grow(a);
}

recursive :: () {
    a: [..] int;

    array_add(*a, 1);
    p := *a[0];

    left(*a, 2);
    sink(p);
}

level1 :: (a: *[..] int) {
    grow(a);
}

level2 :: (a: *[..] int) {
    level1(a);
}

level3 :: (a: *[..] int) {
    level2(a);
}

level4 :: (a: *[..] int) {
    level3(a);
}

level5 :: (a: *[..] int) {
    level4(a);
}

level6 :: (a: *[..] int) {
    level5(a);
}

level7 :: (a: *[..] int) {
    level6(a);
}

level8 :: (a: *[..] int) {
    level7(a);
}

level9 :: (a: *[..] int) {
    level8(a);
}

long_chain :: () {
    a: [..] int;

    array_add(*a, 1);
    p := *a[0];

    level9(*a);
    sink(p);
}

cut :: (a: *[..] int, flag: bool) {
    if flag grow(a);
    else wrapper(a);
}

summary_cut :: (flag: bool) {
    a: [..] int;

    array_add(*a, 1);
    p := *a[0];

    cut(*a, flag);
    sink(p);
}

main :: () {}
