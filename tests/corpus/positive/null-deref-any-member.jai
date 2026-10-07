// Boxing a place behind a null pointer into an `Any` (here a `..Any` argument) reads nothing:
// the callee gets the type and a null value pointer, and only stops if it reads the value.
// jaison passes `(.*) table_find_pointer(...)` to an `assert` that holds, so the pointer is null
// whenever the arguments are boxed.
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
    assert(p == null, "p was %", p.*);
    print("done\n");
}
