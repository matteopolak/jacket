#import "Basic";

main :: () {
    node: Stack_Trace_Node;
    node.next = *node;
    pack_stack_trace(*node);
}
