// NewArray and array_resize clear types without an initializer in bulk and run the
// initializer of the others, element by element.
#import "Basic";

Plain :: struct {
    a: u8;
    b: s64;
}

Defaulted :: struct {
    a: u8 = 7;
    b: s64 = -3;
}

main :: () {
    // Dirty the heap first so a missing clear would show.
    junk := NewArray(64, u8, initialized = false);
    for *junk it.* = 0xAB;
    free(junk.data);

    bytes := NewArray(100, u8);
    for bytes assert(it == 0);
    plain := NewArray(10, Plain);
    for plain assert(it.a == 0 && it.b == 0);
    defaulted := NewArray(10, Defaulted);
    for defaulted assert(it.a == 7 && it.b == -3);

    grow: [..] Plain;
    array_add(*grow, .{ a = 1, b = 2 });
    array_resize(*grow, 5);
    assert(grow.count == 5 && grow[0].b == 2);
    for 1..4 assert(grow[it].a == 0 && grow[it].b == 0);
    array_resize(*grow, 2);
    assert(grow.count == 2);

    with_defaults: [..] Defaulted;
    array_resize(*with_defaults, 3);
    for with_defaults assert(it.a == 7 && it.b == -3);
    print("ok\n");
}
