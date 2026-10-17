// Native ownership oracle for the public lifecycle's serialized context handoff.
#import "Basic";
D :: #import "Debug";

native :: #foreign_library "../../../internal/developer-tools/debug-native/developer_tools_debug";
Callback :: #type (trace: *string, count: s64, data: *void) #c_call;
install :: (callback: Callback, data: *void, count: s64) -> s32 #foreign native "developer_debug_enable_handlers";
oracle :: #foreign_library "../../../../target/developer_tools_debug_lifecycle";
raise_trap :: () -> s32 #foreign oracle "original_debug_raise_trap";

received :: (trace: *string, count: s64, data: *void) #c_call {
    push_context {
        if !data || count != 1 D.abort(.Failure);
        if <<(cast(*u64) data) != 0x123456789abcdef0 D.abort(.Failure);
        D.output("original owned context callback\n");
    }
}

main :: () {
    snapshot: u64 = 0x123456789abcdef0;

    assert(install(received, *snapshot, size_of(u64)) == 1);
    snapshot = 0;
    raised := raise_trap();
    if raised == 0 D.abort(.Failure);
}
