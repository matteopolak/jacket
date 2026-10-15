#import "Basic";
Child :: #import "Measured_Child";

handled_assertions: s64;

recover_assertion :: (loc: Source_Code_Location, message: string) -> bool {
    assert(message == "original recoverable assertion 42");
    handled_assertions += 1;
    return true;
}

main :: () {
    push_context {
        context.assertion_failed = recover_assertion;
        assert(assert_helper("original recoverable assertion %", 42));
    }

    assert(handled_assertions == 1);
    memory := cast(*u8) alloc(8);
    memory[0] = 42;
    assert(memory[0] == Child.child_call());
    free(memory);
    print("ok\n");
}
