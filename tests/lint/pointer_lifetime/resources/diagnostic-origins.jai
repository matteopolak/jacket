#import "Basic";

Node :: struct { value: int; }

snapshot_pair :: () -> *Node, *Node {
    return New(Node), New(Node);
}

aliases :: () {
    original := New(Node);
    alias := original;
    free(original);
    print("%\n", alias.value);
}

double_alias :: () {
    original := New(Node);
    copy := original;
    free(original);
    free(copy);
}

multiple :: () {
    first, second := snapshot_pair();
    first_alias := first;
    second_alias := second;
    free(first);
    free(second);
    print("%\n", first_alias.value);
    print("%\n", second_alias.value);
}

main :: () {
    aliases();
    double_alias();
    multiple();
}
