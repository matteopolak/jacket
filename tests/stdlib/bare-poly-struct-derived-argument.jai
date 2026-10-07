// rules: poly.15 poly.16
// A parameter typed with a bare polymorphic struct accepts any instance, and a struct that has
// an instance as an `#as` member keeps its own type in the body.
#import "Basic";

Base :: struct($Reading: bool) {
    depth: int;
}

Writer :: struct {
    #as using b: Base(false);
    buffer: int;
}

Reader :: struct {
    #as using b: Base(true);
    head: int;
}

step :: (r: *Base) {
    #if r.Reading {
        r.head += 1;
    } else {
        r.buffer += 2;
    }
    r.depth += 1;
}

deepen :: (r: *Base) {
    r.depth += 10;
}

depth_of :: (r: Base) -> int {
    return r.depth;
}

// The `#as` match costs a conversion: an overload taking the argument's own type wins.
Item :: struct {
    x: int;
}

visit :: (d: *$T, r: *Base) -> string {
    return "generic";
}

visit :: (d: *Item, r: *$R) -> string {
    return "custom";
}

main :: () {
    w: Writer;
    step(*w);
    step(*w);
    rd: Reader;
    step(*rd);
    plain: Base(true);
    deepen(*plain);
    deepen(*w);
    assert(w.buffer == 4 && w.depth == 12);
    assert(rd.head == 1 && rd.depth == 1);
    assert(plain.depth == 10);
    assert(depth_of(w) == 12 && depth_of(*rd) == 1);
    item: Item;
    f: float;
    assert(visit(*item, *w) == "custom" && visit(*f, *w) == "generic");
    print("ok\n");
}
