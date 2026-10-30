#import "Basic";

Node :: struct(T: Type) {
    next: *Node(T);
    value: T = 19;
}

answer :: () -> int {
    return 23;
}

main :: () {
    node: Node(int);
    exit(cast(s32) (node.value + #run answer()));
}
