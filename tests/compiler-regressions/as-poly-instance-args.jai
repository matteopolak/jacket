// A struct whose `#as` member is a polymorphic struct instance matches a bare
// polymorphic struct parameter (`*Base`) as that instance.
#import "Basic";

Base :: struct(T: Type) {
    value: T;
}

Derived :: struct {
    using #as base: Base(int);
    extra: int;
}

get :: (b: *Base) -> int {
    return xx b.value;
}

main :: () {
    d: Derived;
    d.value = 41;
    d.extra = 1;
    assert(get(*d) + d.extra == 42);
}

value_of :: (b: *Base($T)) -> T {
    return b.value;
}

#run {
    d: Derived;
    d.value = 5;
    assert(value_of(*d) == 5);
}
