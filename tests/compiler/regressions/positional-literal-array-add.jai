// A positional struct literal that leaves off trailing defaulted members still works when its
// type comes from a polymorphic parameter (`array_add`) or an ordinary procedure argument.
#import "Basic";

Span :: struct {
    start: int;
    end: int;
    kind: int;
    weight := 7;
}

total :: (span: Span) -> int {
    return span.start + span.end + span.kind + span.weight;
}

main :: () {
    spans: [..] Span;

    array_add(*spans, .{ 1, 2, 3 });
    array_add(*spans, .{ 4, 5, 6, 8 });

    assert(spans.count == 2);
    assert(spans[0].weight == 7);
    assert(spans[1].weight == 8);
    assert(total(.{ 1, 2, 3 }) == 13);

    print("ok\n");
}
