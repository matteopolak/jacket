#import "Basic";

Node :: struct {
    next: *Node;
    value: s32;
}

// Low-bit tags work at any pointer width.
low_bits :: (node: *Node) -> u64 {
    return (cast(u64) node) & 7;
}

aligned :: (node: *Node) -> u64 {
    return (cast(u64) node) & ~cast(u64) 7;
}

shifted :: (node: *Node) -> u64 {
    return (cast(u64) node) >> 3;
}

// Plain integers may use all 64 bits.
mix :: (value: u64) -> u64 {
    return (value >> 48) | 0x8000_0000_0000_0000;
}

as_bytes :: (slot: **Node) -> *u8 {
    return cast(*u8) slot;
}

main :: () {
    node: Node;
    print("%\n", low_bits(*node) + mix(1));
}
