#import "Basic";

Node :: struct {
    next: *Node;
}

// A macro may be expanded where `null` would not convert.
zero :: ($T: Type) -> T #expand {
    return cast(T) 0;
}

nothing :: () -> *u8 #expand {
    return cast(*u8) 0;
}

main :: () {
    node: Node;
    node.next = null;
    p: *u8 = null;
    address := cast(u64) 0;
    n := zero(int);
    m := nothing();
    other := cast(*Node) address;
    base := cast(*u8) 4096;

    if node.next == null print("end\n");

    print("% % % % % %\n", p, address, n, m, other, base);
}
