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
}

Registry :: struct {
    names: [..] string;
    label: string;
    count: int;
}

drop_label :: (registry: *Registry) {
    free(registry.label);
}

// Reaches the storage through another helper.
drop_all :: (registry: *Registry) {
    drop_label(registry);
    array_reset(*registry.names);
}

Block :: struct {
    storage: *void;
    allocator: Allocator;
}

// Returns the block's storage through its own allocator.
drop_block :: (block: *Block) {
    block.allocator.proc(.FREE, 0, 0, block.storage, block.allocator.data);
}

label_after_drop :: () {
    registry := New(Registry);
    registry.label = copy_string("kept");

    drop_all(registry);
    print("%\n", registry.label);
}

label_after_drop_on_stack :: () {
    registry: Registry;
    registry.label = copy_string("kept");

    drop_all(*registry);
    print("%\n", registry.label);
}

storage_after_drop :: () {
    block := New(Block);
    block.allocator = context.allocator;
    block.storage = alloc(16);

    drop_block(block);
    print("%\n", block.storage);
}

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
