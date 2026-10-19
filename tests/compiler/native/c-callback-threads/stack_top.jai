// A #c_call procedure that C calls from the very top of a stack: its arguments are all in
// registers, and nothing past them is readable.
#import "Basic";

stack_top :: #library "libstack_top";

Callback :: #type (x: s32) -> s32 #c_call;

call_at_stack_top :: (cb: Callback, x: s32) -> s32 #foreign stack_top;

twice :: (x: s32) -> s32 #c_call {
    return x * 2;
}

main :: () {
    print("from the stack top: %\n", call_at_stack_top(twice, 21));
}
