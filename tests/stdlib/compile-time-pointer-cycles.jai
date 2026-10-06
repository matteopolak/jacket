#import "Basic";

// A compile-time value whose pointers form a cycle is copied into the program once per
// block, with the pointers between the copies intact. (Found by fuzzing: freezing the value
// followed the cycle until the compiler's stack overflowed.)

Node :: struct {
    next: *Node;
    value: int;
}

make_ring :: () -> *Node {
    a := New(Node);
    b := New(Node);
    a.value = 1;
    b.value = 2;
    a.next = b;
    b.next = a;
    return a;
}

Self_Pointing :: struct {
    self: *Self_Pointing;
    value: int;
}

make_self :: () -> Self_Pointing {
    s: Self_Pointing;
    s.self = New(Self_Pointing);
    s.self.self = s.self;
    s.self.value = 7;
    return s;
}

RING :: #run make_ring();
SELF :: #run make_self();

main :: () {
    assert(RING.value == 1 && RING.next.value == 2);
    assert(RING.next.next == RING);
    s := SELF;
    assert(s.self.self == s.self && s.self.value == 7);
    print("ok\n");
}
