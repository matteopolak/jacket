// A positional struct literal that leaves off a member with no default cannot be the item
// of `array_add`; the error says which member has no value.
#import "Basic";

Span :: struct {
    start: int;
    end: int;
    kind: int;
    weight: int;
}

main :: () {
    spans: [..] Span;
    array_add(*spans, .{ 1, 2, 3 });
}
