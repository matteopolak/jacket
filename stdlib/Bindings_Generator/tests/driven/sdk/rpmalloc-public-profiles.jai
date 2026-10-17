// Reviewed rpmalloc.md option profiles and independent allocator ownership.
// Statistics-disabled behavior comes from a real separately compiled C SDK.
#import "Basic";
Default :: #import "rpmalloc";
Rewrite :: #import "rpmalloc"(ENABLE_STATISTICS = false);
Raw :: #import "rpmalloc"(USE_RAW = true, ENABLE_STATISTICS = false);

denied_maps: s64;
denied_unmaps: s64;
denied_retries: s64;

deny_map :: (size: u64, offset: *u64) -> *void {
    assert(size > 0 && offset != null);
    denied_maps += 1;
    return null;
}

deny_unmap :: (address: *void, size: u64, offset: u64, release: u64) {
    denied_unmaps += 1;
}

deny_retry :: (size: u64) -> s32 {
    assert(size > 0);
    denied_retries += 1;
    return 0;
}

main :: () {
    assert(Default.initialize() == 0);
    assert(Rewrite.initialize() == 0);
    // Packaging may share a guarded SDK or select a separate raw heap domain.
    // In either case an overlapping import cannot invalidate Rewrite's owner.
    raw_result := Raw.initialize();

    assert((raw_result == 0) == Raw.is_initialized());
    assert(Rewrite.is_initialized());
    Raw.finalize();
    assert(Rewrite.is_initialized());
    a := Default.rpmalloc(512);
    b := Rewrite.rpmalloc(1024);

    assert(a != null && b != null && a != b);
    (cast(*u8) a)[0] = 31;
    (cast(*u8) b)[0] = 73;
    b = Rewrite.rprealloc(b, 4096);
    assert(b != null && (cast(*u8) b)[0] == 73);
    assert((cast(*u8) a)[0] == 31);

    guarded: struct {
        before: u64 = 0x716;
        stats: Rewrite.rpmalloc_thread_statistics_t;
        after: u64 = 0x918;
    };

    Rewrite.thread_statistics(*guarded.stats);
    assert(guarded.before == 0x716 && guarded.after == 0x918);
    Rewrite.rpfree(b);
    Rewrite.finalize();
    assert(Default.is_initialized() && (cast(*u8) a)[0] == 31);
    input: Raw.rpmalloc_config_t;
    input.memory_map = deny_map;
    input.memory_unmap = deny_unmap;
    input.map_fail_callback = deny_retry;
    assert(Raw.initialize_config(*input) != 0);
    assert(!Raw.is_initialized());
    assert(denied_maps > 0 && denied_unmaps == 0 && denied_retries == 0);
    assert(Default.is_initialized() && (cast(*u8) a)[0] == 31);
    assert(Raw.initialize() == 0);
    b = Raw.rpcalloc(32, 8);
    assert(b != null);
    for 0..255 assert((cast(*u8) b)[it] == 0);

    assert((cast(*u8) a)[0] == 31);
    Raw.rpfree(b);
    Raw.finalize();
    Default.rpfree(a);
    Default.finalize();
}
