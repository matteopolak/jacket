// test: should_fail("a stack trace must not contain a cycle")
#import "Basic";

main :: () {
    node: Stack_Trace_Node;
    node.next = *node;
    pack_stack_trace(*node);
}
