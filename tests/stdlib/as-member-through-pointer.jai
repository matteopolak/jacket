// A pointer to a struct with an `#as` member passes as that member's value.
#import "Basic";

Base :: struct { x: int; }
Derived :: struct { pad: int; #as using base: Base; }

get_x :: (using b: Base) -> int { return x; }

main :: () {
    d: Derived;
    d.x = 42;
    if get_x(*d) != 42 exit(1);
    if get_x(d) != 42 exit(2);
    print("ok\n");
}

// Auto-dereference: `*Thing` passes and returns as `Thing`.
Thing :: struct { v: int; }
thing_v :: (t: Thing) -> int { return t.v; }
deref_return :: (p: *Thing) -> Thing { return p; }

#run {
    t := Thing.{5};
    assert(thing_v(*t) == 5);
    assert(deref_return(*t).v == 5);
}

// Auto-dereference through a polymorphic restriction: `$B/Box` given `*Box(int)`.
Box :: struct (T: Type) { v: T; }
box_value :: (b: $B/Box) -> B.T { return b.v; }

#run {
    b := Box(int).{9};
    assert(box_value(*b) == 9);
}
