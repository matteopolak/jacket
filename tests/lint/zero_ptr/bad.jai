#import "Basic";

Node :: struct {
    next: *Node;
}

global: *Node = cast(*Node) 0;

main :: () {
    node: Node;
    node.next = cast(*Node) 0;
    p := cast(*u8) 0;
    q := cast(*void) 0x0;

    if node.next == cast(*Node) 0 print("end\n");

    print("% % %\n", p, q, global);
}
