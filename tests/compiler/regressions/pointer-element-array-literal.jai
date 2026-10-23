// `*Node.[a, b]` is an array literal whose element type is `*Node`.
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

    print("ok\n");
}
