// A `member = value;` statement in a struct body overrides the default of a
// member reached through a `using` field.
#import "Basic";

Kind :: enum { A; B; C; }
Base :: struct { kind: Kind; extra: int = 7; }
Derived :: struct {
    #as using base: Base;
    kind = .B;
    value: int;
}

main :: () {
    d: Derived;
    n := New(Derived);
    assert(d.kind == .B);
    assert(n.kind == .B);
    assert(d.extra == 7);
}
