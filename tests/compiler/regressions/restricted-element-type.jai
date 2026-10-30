// rules: poly.21
// `$T/Struct` nested in an array or slice element type does not require the argument to
// derive from the struct.
#import "Basic";

Base :: struct {
    t: float;
}

Plain :: struct {
    t: float;
}

Derived :: struct {
    using base: Base;
    extra: int;
}

count_values :: (items: [] $T/Base) -> int {
    return items.count;
}

count_pointers :: (items: [] *$T/Base) -> int {
    return items.count;
}

main :: () {
    plain: [2] Plain;
    derived: [3] Derived;
    pointers: [4] *Plain;

    assert(count_values(plain) == 2);
    assert(count_values(derived) == 3);
    assert(count_pointers(pointers) == 4);
}
