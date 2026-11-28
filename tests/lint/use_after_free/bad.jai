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

Inner :: struct {
    data: *s32;
    scroll: s32;
}

Outer :: struct {
    id: s64;
    inner: Inner;
}

drop_inner_data :: (inner: *Inner) {
    free(inner.data);
}

drop_outer_data :: (outer: *Outer) {
    drop_inner_data(*outer.inner);
}

nested_member_after_drop :: () {
    outer := New(Outer);
    outer.inner.data = New(s32);

    drop_outer_data(outer);
    print("%\n", outer.inner.data);
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

Slot :: struct {
    p: *int;
}

swap_in :: (h: *Slot) {
    free(h.p);
    h.p = New(int);
}

// A call that frees a field and stores a new value in it freed what the old pointer names.
swapped_old_pointer :: () {
    h: Slot;
    h.p = New(int);
    old := h.p;

    swap_in(*h);
    print("%\n", old.*);
    free(h.p);
}

refresh :: (h: *Slot) {
    swap_in(h);
}

// The wrapper's free and replacement are the callee's, in the callee's order.
refreshed_old_pointer :: () {
    h: Slot;
    h.p = New(int);
    old := h.p;

    refresh(*h);
    print("%\n", old.*);
    free(h.p);
}

Cell :: struct {
    storage: *int;
}

peek :: (cell: *Cell) -> int {
    return cell.storage.*;
}

install_and_release :: (cell: *Cell, value: *int) {
    cell.storage = value;
    peek(cell);
    free(cell.storage);
}

// A call that writes nothing in between leaves `cell.storage` holding `value`, which is freed.
installed_then_freed :: () {
    value := New(int);
    cell: Cell;

    install_and_release(*cell, value);
    print("%\n", value.*);
}
