// Variants that contain themselves through a pointer: a case struct holds `[..] *Tree`, so the
// variant is declared before the case it is made of.
//
//     jaic stdlib/Extensions/Variant/tests/recursive.jai -interp

#import "Basic";
#import "Extensions/Variant";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

Tree :: Variant(Leaf, Branch);

Leaf :: struct {
    value: s64;
}

Branch :: struct {
    children: [..] *Tree;
}

// A JSON-like value: a number, text, or a list of more values.
Node :: Variant(s64, string, List);

List :: struct {
    items: [..] *Node;
}

leaf :: (value: s64) -> *Tree {
    result := New(Tree);

    set(result, Leaf.{ value });

    return result;
}

sum :: (tree: *Tree) -> s64 {
    if is(tree.*, Leaf) return unwrap(tree.*, Leaf).value;

    total: s64;
    branch := get_pointer(tree, Branch);

    for branch.children total += sum(it);

    return total;
}

depth :: (node: *Node) -> s64 {
    list := get_pointer(node, List);

    if !list return 0;

    deepest: s64;

    for list.items deepest = max(deepest, depth(it));

    return deepest + 1;
}

tree :: () {
    root := New(Tree);
    branch: Branch;

    array_add(*branch.children, leaf(1));
    array_add(*branch.children, leaf(2));

    inner := New(Tree);
    nested: Branch;

    array_add(*nested.children, leaf(10));
    array_add(*nested.children, leaf(20));
    set(inner, nested);
    array_add(*branch.children, inner);
    set(root, branch);

    check(sum(root) == 33, "a tree sums through its pointers");
    check(is(root.*, Branch) && !is(root.*, Leaf), "the root is a branch");
    check(size_of(Tree) >= size_of(Branch), "the payload holds the largest case");
    check(variant_type(root.*) == Branch, "variant_type of a recursive variant");

    free_variant(root);
}

nodes :: () {
    a := New(Node);
    b := New(Node);
    c := New(Node);

    set(a, 5);
    set(b, "text");

    list: List;

    array_add(*list.items, a);
    array_add(*list.items, b);
    set(c, list);

    outer: List;

    array_add(*outer.items, c);

    root := variant(Node, outer);

    check(depth(*root) == 2, "lists nest through pointers");
    check(unwrap(a.*, s64) == 5, "a number inside a list");
    check(unwrap(b.*, string) == "text", "text inside a list");
    check(root == root, "a recursive variant equals itself");
}

main :: () {
    tree();
    nodes();

    if failures print("% check(s) failed\n", failures);
    else print("ok\n");
}
