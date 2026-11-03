#import "Basic";

Node :: struct {
    value: int;
}

main :: () {
    node := New(Node);
    node.value = 7;

    free(node);
    print("%\n", node.value);
}
