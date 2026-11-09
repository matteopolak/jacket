#import "Basic";

Node :: struct {
    value: int;
}

Registry :: struct {
    item: *Node;
    other: *Node;
}

keep :: (r: *Registry, n: *Node) {
    r.item = n;
}

keep_twice :: (r: *Registry, n: *Node) {
    keep(r, n);
}

keep_other :: (r: *Registry, n: *Node) {
    r.other = n;
}

keep_if :: (r: *Registry, n: *Node, c: bool) {
    if c r.item = n;
}

keep_then_clear :: (r: *Registry, n: *Node) {
    r.item = n;
    r.item = null;
}

freed_after_keep :: () {
    n := New(Node);
    r: Registry;

    keep(*r, n);
    free(n);
    print("%\n", r.item.value);
}

freed_after_keep_twice :: () {
    n := New(Node);
    r: Registry;

    keep_twice(*r, n);
    free(n);
    print("%\n", r.item.value);
}

kept_in_other_field :: () {
    n := New(Node);
    r: Registry;

    keep_other(*r, n);
    free(n);
    print("%\n", r.other.value);
}

other_field_unaffected :: () {
    n := New(Node);
    m := New(Node);
    r: Registry;

    keep(*r, m);
    keep_other(*r, n);
    free(n);
    print("%\n", r.item.value);
    free(m);
}

replaced_after_keep :: () {
    n := New(Node);
    m := New(Node);
    r: Registry;

    keep(*r, n);
    keep(*r, m);
    free(n);
    print("%\n", r.item.value);
    free(m);
}

kept_on_some_paths :: () {
    n := New(Node);
    r: Registry;
    r.item = New(Node);
    keep_if(*r, n, false);
    free(n);
    print("%\n", r.item.value);
    free(r.item);
}

stored_then_cleared :: () {
    n := New(Node);
    r: Registry;

    keep_then_clear(*r, n);
    free(n);
    print("%\n", r.item == null);
}

kept_local_address :: (r: *Registry) {
    n: Node;
    keep(r, *n);
}

kept_local_address_twice :: (r: *Registry) {
    n: Node;
    keep_twice(r, *n);
}

kept_in_local_registry :: () {
    r: Registry;
    n: Node;
    keep(*r, *n);
    print("%\n", r.item.value);
}

kept_heap_in_parameter :: (r: *Registry) {
    n := New(Node);
    keep(r, n);
}

Labelled :: struct {
    label: string;
    other: string;
}

keep_label :: (l: *Labelled, text: string) {
    l.label = text;
}

keep_label_twice :: (l: *Labelled, text: string) {
    keep_label(l, text);
}

freed_label_after_keep :: () {
    text := copy_string("label");
    l: Labelled;

    keep_label(*l, text);
    free(text);
    print("%\n", l.label);
}

freed_label_after_keep_twice :: () {
    text := copy_string("label");
    l: Labelled;

    keep_label_twice(*l, text);
    free(text);
    print("%\n", l.label);
}

label_other_field_unaffected :: () {
    text := copy_string("label");
    l: Labelled;

    keep_label(*l, "kept");
    free(text);
    print("%\n", l.label);
}

main :: () {}
