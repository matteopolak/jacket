#import "Basic";

Node :: struct {
    next: *Node;
    value: s32;
}

// A 16-bit tag in the top of the pointer.
tag_of :: (node: *Node) -> u64 {
    return (cast(u64) node) >> 48;
}

main :: () {
    node: Node;
    print("%\n", tag_of(*node));
}
