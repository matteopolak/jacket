#import "Basic";

Node :: struct { value: int; }

replaced_origin :: () {
    node := New(Node);
    free(node);
    node = New(Node);
    node.value = 9;
    replacement_alias := node;
    free(node);
    print("%\n", replacement_alias.value);
}

incoming_origin :: (argument: *Node) {
    free(argument);
    print("%\n", argument.value);
}

branch_origin :: (flag: bool) {
    joined: *Node;
    if flag joined = New(Node);
    else joined = New(Node);
    free(joined);
    print("%\n", joined.value);
}

captured_before_origin :: () {
    saved_origin := New(Node);
    captured_old := saved_origin;
    saved_origin = New(Node);
    saved_origin.value = 3;
    free(captured_old);
    print("%\n", captured_old.value);
    free(saved_origin);
}

rewrite_origin :: (slot: **Node, replacement: *Node) {
    slot.* = replacement;
}

exposed_origin :: () {
    exposed := New(Node);
    rewrite_origin(*exposed, New(Node));
    free(exposed);
    print("%\n", exposed.value);
}

main :: () {
    replaced_origin();
    incoming_origin(New(Node));
    branch_origin(true);
    captured_before_origin();
    exposed_origin();
}
