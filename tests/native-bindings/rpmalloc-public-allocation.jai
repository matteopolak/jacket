// Oracle: docs/specs/stdlib/modules/rpmalloc.md allocation ownership contract.
// Independently authored public lifecycle, allocation and statistics tests.
#import "Basic";
RPM :: #import "rpmalloc";

main :: () {
    assert(!RPM.is_initialized());
    assert(RPM.initialize() == 0);
    assert(RPM.is_initialized());
    RPM.thread_initialize();
    assert(RPM.is_thread_initialized());
    memory := RPM.rpmalloc(173);

    assert(memory != null);
    bytes := cast(*u8) memory;
    for 0..172 bytes[it] = cast(u8) it;

    // Independent safety checks: a rejected signed request must not consume
    // the original allocation or enter the unsigned native API.
    assert(RPM.rpmalloc(-1) == null);
    assert(RPM.rprealloc(memory, -1) == null);
    for 0..172 assert(bytes[it] == cast(u8) it);

    assert(RPM.usable_size(memory) >= 173);
    grown := RPM.rprealloc(memory, 8192);

    assert(grown != null);
    bytes = cast(*u8) grown;
    for 0..172 assert(bytes[it] == cast(u8) it);

    RPM.rpfree(grown);
    aligned := RPM.rpaligned_alloc(256, 301);

    assert(aligned != null);
    assert(cast(u64) aligned % 256 == 0);
    bytes = cast(*u8) aligned;
    for 0..300 bytes[it] = 91;

    resized := RPM.rpaligned_realloc(aligned, 256, 16384, 301, .{});

    assert(resized != null);
    assert(cast(u64) resized % 256 == 0);
    bytes = cast(*u8) resized;
    for 0..300 assert(bytes[it] == 91);

    RPM.rpfree(resized);
    RPM.rpfree(null);

    guarded: struct {
        before: u64 = 0x93ab;
        stats: RPM.rpmalloc_thread_statistics_t;
        after: u64 = 0x9876;
    };

    RPM.thread_statistics(*guarded.stats);
    assert(guarded.before == 0x93ab && guarded.after == 0x9876);
    allocations: s64;
    frees: s64;

    for guarded.stats.size_use {
        allocations += it.alloc_total;
        frees += it.free_total;
        assert(it.alloc_peak >= it.alloc_current);
    }

    assert(allocations > 0 && frees > 0);
    globals: RPM.rpmalloc_global_statistics_t;

    RPM.global_statistics(*globals);
    assert(globals.mapped_total > 0);
    assert(globals.mapped_peak >= globals.mapped);
    RPM.thread_finalize(true);
    assert(!RPM.is_thread_initialized());
    RPM.finalize();
    assert(!RPM.is_initialized());
}
