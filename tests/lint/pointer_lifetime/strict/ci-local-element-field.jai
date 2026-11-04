#import "Basic";

Node :: struct {
    child: *Node;
}

local_children :: () {
    leaf: Node;
    nodes: [3] Node;
    copies: [..] Node;

    for *nodes {
        it.child = *leaf;
        array_add(*copies, it.*);
    }

    assert(copies[0].child == *leaf);
    array_free(copies);
}

durable: Node;

bad :: () {
    leaf: Node;
    durable.child = *leaf;
}

inner_leaf :: () {
    nodes: [3] Node;

    {
        leaf: Node;
        for *nodes it.child = *leaf;
    }

    assert(nodes[0].child != null);
}
