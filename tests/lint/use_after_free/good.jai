#import "Basic";

Node :: struct {
    value: int;
    next: *Node;
}

report :: (node: *Node) {
    print("node %\n", node.value);
}

// Not a plain variable: the call frees something the variable holds.
free_next :: (node: *Node) {
    free(node.next);
    node.next = null;
}

Holder :: struct {
    name: string;
}

// Releases what the holder owns; the holder itself stays valid.
release_holder :: (holder: *Holder) {
    free(holder.name);
} @FreesArgument

restart :: (holder: *Holder) {
    release_holder(holder);
    holder.* = .{};
    free(holder);
}

main :: () {
    // A new value ends the freed one.
    node := New(Node);

    free(node);
    node = New(Node);
    report(node);

    // Cleared right after the free, and compared with `null`.
    other := New(Node);

    free(other);
    if other == null print("gone\n");

    other = null;

    // The branch that frees leaves before the later use.
    third := New(Node);
    if third.value == 0 {
        free(third);
        return;
    }

    report(third);

    // A pointer to the variable may change it behind the scan's back.
    fourth := New(Node);
    slot := *fourth;

    free(fourth);
    slot.* = New(Node);
    report(fourth);

    // Freed at the end of the scope: never read afterwards.
    fifth := New(Node);

    defer free(fifth);
    report(fifth);
}
