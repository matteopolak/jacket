// Distinct thread contexts must be published with the serialized native install.
#import "Basic";
D :: #import "Debug";

#add_context original_debug_sentinel: u64;
#add_context original_debug_complement: u64;

oracle :: #foreign_library "../../../../target/developer_tools_debug_lifecycle";
Work :: #type (tag: s64) #c_call;
run_threads :: (work: Work) -> s32 #foreign oracle "original_debug_run_threads";
install_prior :: () -> s32 #foreign oracle "original_debug_install_prior";
changed_actions :: () -> s32 #foreign oracle "original_debug_changed_actions";
prior_restored :: () -> s32 #foreign oracle "original_debug_prior_restored";
raise_trap :: () -> s32 #foreign oracle "original_debug_raise_trap";

work :: (tag: s64) #c_call {
    push_context {
        context.original_debug_sentinel = cast(u64) tag;
        context.original_debug_complement = ~context.original_debug_sentinel;

        for 0..199 {
            D.enable_signal_handler();
            D.disable_signal_handler();
        }
    }
}

on_crash :: (trace: [] string) {
    if context.original_debug_sentinel != 0x12345678 D.abort(.Failure);
    if context.original_debug_complement != ~context.original_debug_sentinel D.abort(.Failure);
    if trace.count != 1 D.abort(.Failure);
    D.output("original concurrent context callback\n");
}

main :: () {
    assert(install_prior() == 1);
    assert(run_threads(work) == 1);
    D.disable_signal_handler();
    assert(prior_restored() == 1);
    context.original_debug_sentinel = 0x12345678;
    context.original_debug_complement = ~context.original_debug_sentinel;
    D.crash_handler = on_crash;
    D.enable_signal_handler();
    assert(changed_actions() == 5);
    context.original_debug_sentinel = 0;
    context.original_debug_complement = 0;
    raised := raise_trap();
    if raised == 0 D.abort(.Failure);
}
