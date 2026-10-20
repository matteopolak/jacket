#import "Basic";

Node :: struct {
    value: int;
    next: *Node;
}

report :: (node: *Node) {
    print("node %\n", node.value);
}

Holder :: struct {
    name: string;
}

// Releases what the holder owns; the holder itself stays valid.
release_holder :: (holder: *Holder) {
    free(holder.name);
} @FreesArgument

rename :: (holder: *Holder) {
    release_holder(holder);
    print("was %\n", holder.name);
}

main :: () {
    node := New(Node);
    node.value = 3;

    free(node);
    print("freed node held %\n", node.value);

    text := copy_string("hello");

    free(text);
    print("%\n", text);

    other := New(Node);

    free(other);
    report(other);
}
