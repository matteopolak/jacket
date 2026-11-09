#import "Basic";

Node :: struct {
    value: int;
}

Holder :: struct {
    node: *Node;
    other: *Node;
}

same :: (n: *Node) -> *Node {
    return n;
}

same_twice :: (n: *Node) -> *Node {
    return same(n);
}

fresh :: () -> *Node {
    return New(Node);
}

fresh_twice :: () -> *Node {
    return fresh();
}

field_of :: (h: *Holder) -> *Node {
    return h.node;
}

other_of :: (h: *Holder) -> *Node {
    return h.other;
}

pick :: (a: *Node, b: *Node) -> *Node {
    return a;
}

alias_freed_source :: () {
    node := New(Node);
    alias := same(node);
    free(node);
    print("%\n", alias.value);
}

alias_freed_through_two :: () {
    node := New(Node);
    alias := same_twice(node);
    free(node);
    print("%\n", alias.value);
}

fresh_is_independent :: () {
    node := New(Node);
    other := fresh();

    free(node);
    print("%\n", other.value);
    free(other);
}

fresh_twice_is_independent :: () {
    node := New(Node);
    other := fresh_twice();

    free(node);
    print("%\n", other.value);
    free(other);
}

alias_of_field :: () {
    holder: Holder;
    holder.node = New(Node);
    holder.other = New(Node);
    alias := field_of(*holder);

    free(holder.node);
    print("%\n", alias.value);
    free(holder.other);
}

other_field_is_independent :: () {
    holder: Holder;
    holder.node = New(Node);
    holder.other = New(Node);
    alias := field_of(*holder);

    free(holder.other);
    print("%\n", alias.value);
    free(holder.node);
}

second_field_alias :: () {
    holder: Holder;
    holder.node = New(Node);
    holder.other = New(Node);
    alias := other_of(*holder);

    free(holder.other);
    print("%\n", alias.value);
    free(holder.node);
}

second_field_independent :: () {
    holder: Holder;
    holder.node = New(Node);
    holder.other = New(Node);
    alias := other_of(*holder);

    free(holder.node);
    print("%\n", alias.value);
    free(holder.other);
}

picks_first :: () {
    a := New(Node);
    b := New(Node);
    alias := pick(a, b);

    free(b);
    print("%\n", alias.value);
    free(a);
}

picks_first_freed :: () {
    a := New(Node);
    b := New(Node);
    alias := pick(a, b);

    free(a);
    print("%\n", alias.value);
    free(b);
}

main :: () {}
