// Reviewed rpmalloc.md: existing diagnostic options, allocation ownership,
// paired callback failure and independent import-instance lifetime.
Default :: #import "rpmalloc";
NoAssertions :: #import "rpmalloc"(ENABLE_ASSERTS = false);
NoValidation :: #import "rpmalloc"(ENABLE_VALIDATE_ARGS = false);
Neither :: #import "rpmalloc"(ENABLE_ASSERTS = false, ENABLE_VALIDATE_ARGS = false);

libc :: #library,system "c";
abort :: () #foreign libc;

#add_context diagnostic_callback_marker: s64;
denied_maps: s64;
denied_unmaps: s64;

check :: (condition: bool) {
    if !condition abort();
}

deny_map :: (size: s64, offset: *s64) -> *void {
    check(context.diagnostic_callback_marker == 713);
    check(size > 0 && offset != null);
    denied_maps += 1;
    return null;
}

deny_unmap :: (address: *void, size: s64, offset: s64, release: s64) {
    denied_unmaps += 1;
}

main :: () {
    check(Default.initialize() == 0);
    allocator_name: string;
    caps := Default.rpmalloc_allocator_proc(.CAPS, 0, 0, cast(*void) *allocator_name, null);
    expected_caps: Allocator_Caps = .HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR | .MULTIPLE_THREADS |
                                    .FREE | .ACTUALLY_RESIZE | .CREATE_HEAP;

    check(cast(u64) caps == cast(u64) expected_caps);
    check(allocator_name == "rpmalloc 1.4.4");
    check(NoAssertions.initialize() == 0);
    check(NoValidation.initialize() == 0);
    check(Default.is_initialized() && NoAssertions.is_initialized() && NoValidation.is_initialized());
    check(cast(*void) Default.config() != cast(*void) NoAssertions.config());
    check(cast(*void) NoValidation.config() != cast(*void) NoAssertions.config());
    baseline := Default.rpmalloc(73);
    a := NoAssertions.rpaligned_alloc(256, 129);
    v := NoValidation.rpmalloc(193);

    check(baseline != null && a != null && v != null);
    check(cast(u64) a % 256 == 0);
    (cast(*u8) baseline)[0] = 19;

    for 0..128 {
        (cast(*u8) a)[it] = 31;
    }
    for 0..192 {
        (cast(*u8) v)[it] = 47;
    }

    // Public signed bounds and safe ABI alignment guards remain meaningful
    // even when the SDK's optional diagnostics have been compiled out.
    check(NoAssertions.rpmalloc(-1) == null);
    check(NoValidation.rprealloc(v, -1) == null);
    check(NoValidation.rpaligned_alloc(3, 32) == null);
    for 0..192 check((cast(*u8) v)[it] == 47);

    grown := NoAssertions.rpaligned_realloc(a, 256, 4096, 129, .{});

    check(grown != null && cast(u64) grown % 256 == 0);
    for 0..128 check((cast(*u8) grown)[it] == 31);

    NoAssertions.rpfree(grown);
    NoAssertions.finalize();
    check(Default.is_initialized() && NoValidation.is_initialized());
    check((cast(*u8) baseline)[0] == 19);
    for 0..192 check((cast(*u8) v)[it] == 47);

    context.diagnostic_callback_marker = 713;
    input: Neither.rpmalloc_config_t;
    input.memory_map = deny_map;
    input.memory_unmap = deny_unmap;
    check(Neither.initialize_config(*input) != 0);
    check(!Neither.is_initialized() && denied_maps > 0 && denied_unmaps == 0);
    check(context.diagnostic_callback_marker == 713);
    check(Default.is_initialized() && NoValidation.is_initialized());
    check(Neither.initialize() == 0);
    n := Neither.rpmalloc(257);

    check(n != null);
    (cast(*u8) n)[256] = 91;
    heap := Neither.heap_acquire();

    check(heap != null);
    owned := Neither.heap_alloc(heap, 8192);

    check(owned != null && Neither.heap_is_this_yours(heap, owned));
    Neither.heap_free(heap, owned);
    Neither.heap_release(heap);
    check((cast(*u8) n)[256] == 91);
    Neither.rpfree(n);
    Neither.finalize();
    NoValidation.rpfree(v);
    NoValidation.finalize();
    check(Default.is_initialized() && (cast(*u8) baseline)[0] == 19);
    Default.rpfree(baseline);
    Default.finalize();
}
