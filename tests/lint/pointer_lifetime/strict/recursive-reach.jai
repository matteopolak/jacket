Node :: struct {
    a: *Node;
    b: *Node;
    c: *Node;
    d: *Node;
    values: [..] int;
}

lib :: #library "c";
opaque :: (node: *Node) #foreign lib "lifetime_fixture_unknown_tree";

sink :: (p: *int) -> int {
    return p.*;
}

visit :: (node: *Node, depth: int) {
    opaque(node);

    if depth > 0 {
        visit(node.a, depth - 1);
        visit(node.b, depth - 1);
        visit(node.c, depth - 1);
        visit(node.d, depth - 1);
    }
}

probe :: () {
    node: Node;
    p := *node.values[0];
    visit(*node, 2);
    sink(p);
}
