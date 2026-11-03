#import "Basic";

Node :: struct {
    value: int;
}

main :: () {
    node := New(Node);
    node.value = 3;
    print("%\n", node.value);

    free(node);
}
