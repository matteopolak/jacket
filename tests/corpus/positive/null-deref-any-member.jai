// A member at offset 0 of a null struct pointer boxed into an `Any` (here a `..Any` argument)
// stops with a null pointer dereference instead of handing the callee a null value pointer.
#import "Basic";

Node :: struct {
    value: int;
    next: *Node;
}

describe :: (values: ..Any) -> int {
    count := 0;
    for values if it.value_pointer count += 1;
    return count;
}

main :: () {
    node := Node.{value = 3};
    p := *node;
    print("%\n", describe(p.value, p.next));
    p = p.next;
    print("%\n", describe(p.value));
    print("unreachable\n");
}
