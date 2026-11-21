#import "Basic";

Node :: struct {
    next: *Node;
    value: s32;
}

// Tags in the top 16 bits of a pointer.
tag_of :: (node: *Node) -> u64 {
    return (cast(u64) node) >> 48;
}

strip_tag :: (node: *Node) -> u64 {
    return (cast(u64) node) & 0x0000_ffff_ffff_ffff;
}

with_tag :: (node: *Node, tag: u64) -> u64 {
    return (cast(u64) node) | 0x8000_0000_0000_0000;
}

// Reads the pointer in `slot` and four bytes after it as one integer.
as_integer :: (slot: **Node) -> *u64 {
    return cast(*u64) slot;
}

fixed_address :: () -> *Node {
    return cast(*Node) cast(u64) 0x1_0000_0000;
}

main :: () {
    node: Node;
    print("%\n", tag_of(*node));
}
