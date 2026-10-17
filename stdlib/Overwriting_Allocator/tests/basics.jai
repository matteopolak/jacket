#import "Basic";
O :: #import "Overwriting_Allocator";
T :: #import "Thread";

Probe :: struct {
    base: Allocator;
    tracked: *void;
    expected_size: s64;
    poison: u8 = 0xa7;
    frees: s64;
    fail_size: s64 = -1;
}

forward :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    probe := cast(*Probe) allocator_data;
    if mode == .ALLOCATE && requested_size == probe.fail_size return null;

    if mode == .FREE && old_memory == probe.tracked {
        assert(old_size == probe.expected_size);
        bytes := cast(*u8) old_memory;
        for i: 0..old_size - 1 assert(bytes[i] == probe.poison);
        probe.frees += 1;
    }

    return probe.base.proc(mode, requested_size, old_size, old_memory, probe.base.data);
}

main :: () {
    probe: Probe;
    probe.base = context.allocator;
    allocator := O.get_overwriting_allocator(.{ forward, *probe }, probe.poison);
    wrapper := cast(*O.Overwriting_Allocator) allocator.data;
    original := cast(*u8) allocator.proc(.ALLOCATE, 17, 0, null, allocator.data);

    assert(original != null && wrapper.table.count == 1);
    for i: 0..16 assert(original[i] == probe.poison);

    memset(original, 42, 17);
    probe.tracked = original;
    probe.expected_size = 17;
    probe.fail_size = 31;
    failed := allocator.proc(.RESIZE, 31, 17, original, allocator.data);

    assert(failed == null && probe.frees == 0 && wrapper.table.count == 1);
    for i: 0..16 assert(original[i] == 42);

    probe.fail_size = -1;
    grown := cast(*u8) allocator.proc(.RESIZE, 32, 17, original, allocator.data);

    assert(grown != null && grown != original && probe.frees == 1 && wrapper.table.count == 1);
    for i: 0..16 assert(grown[i] == 42);
    for i: 17..31 assert(grown[i] == probe.poison);

    probe.tracked = grown;
    probe.expected_size = 32;
    allocator.proc(.FREE, 0, 0, grown, allocator.data);
    assert(probe.frees == 2 && wrapper.table.count == 0);
    caps := cast(Allocator_Caps) cast(u64) allocator.proc(.CAPS, 0, 0, null, allocator.data);

    assert((caps & (.FREE | .MULTIPLE_THREADS)) == (.FREE | .MULTIPLE_THREADS));
    probe.tracked = null;
    threads: [4] T.Thread;
    for *thread: threads {
        thread.data = *allocator;
        assert(T.thread_init(thread, concurrent_allocations));
        T.thread_start(thread);
    }

    for *thread: threads {
        assert(T.thread_is_done(thread, 5000));
        T.thread_deinit(thread);
    }

    assert(wrapper.table.count == 0);
    O.deinit_overwriting_allocator(allocator);
    captured := O.get_overwriting_allocator();
    data := cast(*u8) captured.proc(.ALLOCATE, 1, 0, null, captured.data);

    assert(data != null && data.* == 0xcd);
    captured.proc(.FREE, 0, 0, data, captured.data);
    O.deinit_overwriting_allocator(captured);
    print("ok\n");
}

concurrent_allocations :: (thread: *T.Thread) -> s64 {
    allocator := (cast(*Allocator) thread.data).*;

    for 0..127 {
        memory := cast(*u8) allocator.proc(.ALLOCATE, 64, 0, null, allocator.data);
        assert(memory != null && memory[0] == 0xa7 && memory[63] == 0xa7);
        memset(memory, 17, 64);
        allocator.proc(.FREE, 0, 0, memory, allocator.data);
    }

    return 0;
}
