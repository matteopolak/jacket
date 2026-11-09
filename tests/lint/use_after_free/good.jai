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
}

Registry :: struct {
    label: string;
    count: int;
}

drop_label :: (registry: *Registry) {
    free(registry.label);
}

// Frees the label and gives the field a new value, so the registry is reusable.
reset_label :: (registry: *Registry) {
    free(registry.label);
    registry.label = "";
}

// Frees only a copy of its own.
drop_scratch :: (registry: *Registry) {
    scratch := copy_string("scratch");
    free(scratch);
}

// Frees only on one path.
drop_sometimes :: (registry: *Registry, really: bool) {
    if really free(registry.label);
}

Tree :: struct {
    child: *Tree;
    routes: [..] int;
}

// Frees everything below the node, recursively, but not the node's own members.
free_below :: (node: *Tree) {
    if node.child {
        free_below(node.child);
        array_reset(*node.child.routes);
        free(node.child);
    }
}

tidy_child :: (node: *Tree) {
    if node.child {
        free_below(node.child);
        array_reset(*node.child.routes);
    }
}

other_field_after_drop :: () {
    registry := New(Registry);
    registry.label = copy_string("gone");
    registry.count = 3;

    drop_label(registry);
    print("%\n", registry.count);
}

reused_after_reset :: () {
    registry := New(Registry);
    registry.label = copy_string("gone");

    reset_label(registry);
    print("%\n", registry.label);
}

after_scratch_drop :: () {
    registry := New(Registry);
    registry.label = copy_string("kept");

    drop_scratch(registry);
    print("%\n", registry.label);
}

after_conditional_drop :: (really: bool) {
    registry := New(Registry);
    registry.label = copy_string("maybe");

    drop_sometimes(registry, really);
    print("%\n", registry.label);
}

restart :: (holder: *Holder) {
    release_holder(holder);
    holder.* = .{};
    free(holder);
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

// Frees the storage one level further down: the other members of `inner` stay usable.
drop_outer_data :: (outer: *Outer) {
    drop_inner_data(*outer.inner);
}

nested_member_after_drop :: () {
    outer := New(Outer);
    outer.inner.scroll = 3;

    drop_outer_data(outer);
    outer.inner.scroll = 4;
    print("%\n", outer.inner.scroll);
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
