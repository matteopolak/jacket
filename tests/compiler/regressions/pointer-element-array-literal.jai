// `*Node.[a, b]` is an array literal whose element type is `*Node`; `*(T.[a, b])` takes the
// address of an array literal.
#import "Basic";

Node :: struct {
    index: s64;
    next: *Node;
}

main :: () {
    first := New(Node);

    first.index = 7;

    slots := *Node.[first, first, first];

    assert(type_of(slots) == [3] *Node);
    assert(slots[1].index == 7);

    values := *(float.[1.5, 3.5]);

    assert(type_of(values) == *[2] float);
    assert(values.*[1] == 3.5);
    erased: *void = *(float.[0.2, 1.0]);

    assert((cast(*float) erased).* == 0.2);

    print("ok\n");
}
