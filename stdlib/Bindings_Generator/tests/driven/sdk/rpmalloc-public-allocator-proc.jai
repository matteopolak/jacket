// Reviewed rpmalloc generic mode table. The canonical bootstrap Allocator_Proc
// and Allocator records are used directly; no binding-specific replacement.
#import "Basic";
RPM :: #import "rpmalloc";
Small :: #import "rpmalloc"(ENABLE_STATISTICS = false);
Raw :: #import "rpmalloc"(USE_RAW = true);
RawSmall :: #import "rpmalloc"(USE_RAW = true, ENABLE_STATISTICS = false);

probe :: #library,system "rpmalloc_callback_probe";
parallel :: (worker: #type () #c_call) -> s32 #foreign probe "binding_rpmalloc_parallel";
claim :: () -> u32 #foreign probe "binding_rpmalloc_handoff_claim";
reset_claims :: () #foreign probe "binding_rpmalloc_handoff_reset";
selected: Allocator_Proc;
shared_heap: *void;
handoff: [4] *void;

free_worker :: () #c_call {
    push_context, defer_pop;
    selected(.THREAD_START, 0, 0, null, null);
    index := claim();

    assert(index < 4 && handoff[index] != null);
    assert((cast(*u8) handoff[index])[0] == cast(u8) (81 + index));
    assert(selected(.FREE, 0, 0, handoff[index], shared_heap) == null);
    handoff[index] = null;
    assert(selected(.THREAD_STOP, 0, 0, null, null) == null);
}

exercise :: (proc: Allocator_Proc, initialized: #type () -> bool,
             thread_initialized: #type () -> bool) {
    assert(proc(.STARTUP, 0, 0, null, null) == null && initialized());

    name_guard: struct {
        before: u64 = 0x716;
        name: string;
        after: u64 = 0x918;
    };

    caps := cast(u64) proc(.CAPS, 0, 0, cast(*void) *name_guard.name, null);
    expected := cast(u64) (Allocator_Caps.HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR |
                          Allocator_Caps.MULTIPLE_THREADS | Allocator_Caps.FREE |
                          Allocator_Caps.ACTUALLY_RESIZE | Allocator_Caps.CREATE_HEAP);

    assert(caps == expected);
    assert(name_guard.before == 0x716 && name_guard.after == 0x918);
    assert(name_guard.name.count > 0);
    previous_name := name_guard.name;

    assert(cast(u64) proc(.CAPS, 0, 0, cast(*void) *name_guard.name, null) == caps);
    assert(previous_name.data == name_guard.name.data && previous_name == name_guard.name);
    assert(cast(u64) proc(.CAPS, 0, 0, null, null) == caps);
    heap := proc(.CREATE_HEAP, 0, 0, null, null);
    second := proc(.CREATE_HEAP, 0, 0, null, null);

    assert(heap != null && second != null && heap != second);
    value: Allocator = .{ proc, heap };
    actual_caps, actual_name := get_capabilities(value);

    assert(cast(u64) actual_caps == caps && actual_name == previous_name);
    memory := proc(.ALLOCATE, 301, 0, null, heap);

    assert(memory != null);
    (cast(*u8) memory)[0] = 71;
    assert(cast(u64) proc(.IS_THIS_YOURS, 0, 0, memory, heap) == 1);
    assert(cast(u64) proc(.IS_THIS_YOURS, 0, 0, memory, second) == 0);
    assert(cast(u64) proc(.IS_THIS_YOURS, 0, 0, cast(*void) 123, heap) == 0);
    replacement := proc(.RESIZE, 4096, -999, memory, heap);

    assert(replacement != null && (cast(*u8) replacement)[0] == 71);
    assert(proc(.RESIZE, -1, 0, replacement, heap) == null);
    assert((cast(*u8) replacement)[0] == 71);
    assert(proc(.FREE, 0, 0, replacement, heap) == null);
    full: [4] *void;
    for *full {
        it.* = proc(.ALLOCATE, 32000, 0, null, heap);
        assert(it.* != null);
    }

    for full assert(cast(u64) proc(.IS_THIS_YOURS, 0, 0, it, heap) == 1);
    for full assert(proc(.FREE, 0, 0, it, heap) == null);

    large := proc(.ALLOCATE, 1_048_576, 0, null, heap);
    huge := proc(.ALLOCATE, 33_554_432, 0, null, heap);

    assert(large != null && huge != null);
    assert(cast(u64) proc(.IS_THIS_YOURS, 0, 0, large, heap) == 1);
    assert(cast(u64) proc(.IS_THIS_YOURS, 0, 0, huge, heap) == 1);
    assert(proc(.FREE, 0, 0, large, heap) == null);
    assert(proc(.FREE, 0, 0, huge, heap) == null);
    selected = proc;
    shared_heap = heap;

    for 0..3 {
        handoff[it] = proc(.ALLOCATE, 512, 0, null, heap);
        assert(handoff[it] != null);
        (cast(*u8) handoff[it])[0] = cast(u8) (81 + it);
    }

    reset_claims();
    assert(parallel(free_worker) == 0);
    for handoff assert(it == null);

    assert(proc(.DESTROY_HEAP, 0, 0, null, second) == null);
    assert(proc(.DESTROY_HEAP, 0, 0, null, heap) == null);
    memory = proc(.ALLOCATE, 512, 0, null, null);
    assert(memory != null);
    (cast(*u8) memory)[0] = 61;
    assert(cast(u64) proc(.IS_THIS_YOURS, 0, 0, memory, null) == 1);
    replacement = proc(.RESIZE, 4096, -1, memory, null);
    assert(replacement != null && (cast(*u8) replacement)[0] == 61);
    assert(proc(.FREE, 0, 0, replacement, null) == null);
    assert(proc(.THREAD_STOP, 0, 0, null, null) == null && !thread_initialized());
    assert(proc(.THREAD_START, 0, 0, null, null) == null && thread_initialized());
    assert(proc(.SHUTDOWN, 0, 0, null, null) == null && !initialized());
}

main :: () {
    exercise(RPM.rpmalloc_allocator_proc, RPM.is_initialized, RPM.is_thread_initialized);
    exercise(Small.rpmalloc_allocator_proc, Small.is_initialized, Small.is_thread_initialized);
    exercise(Raw.rpmalloc_allocator_proc, Raw.is_initialized, Raw.is_thread_initialized);
    exercise(RawSmall.rpmalloc_allocator_proc, RawSmall.is_initialized, RawSmall.is_thread_initialized);
}
