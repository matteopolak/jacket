#import "Basic";

Node :: struct {
    next: *Node;
    value: s32;
    tag: u16;
}

// The tag lives next to the pointer.
tag_of :: (node: *Node) -> u16 {
    return node.tag;
}

main :: () {
    node: Node;
    print("%\n", tag_of(*node));
}
