// push_allocator(proc, data) keeps the allocator for the rest of the caller's scope,
// then restores the previous one.
#import "Basic";

Tally :: struct {
    calls: int;
}

tally_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    (cast(*Tally) data).calls += 1;
    return context.default_allocator.proc(mode, size, old_size, old_memory, context.default_allocator.data);
}

main :: () -> int {
    tally: Tally;
    before := context.allocator;
    {
        push_allocator(tally_proc, *tally);
        if context.allocator.proc != tally_proc return 1;
        text := sprint("%", 42);
        free(text);
    }
    if tally.calls < 2 return 2;
    if context.allocator.proc != before.proc return 3;
    print("ok\n");
    return 0;
}
