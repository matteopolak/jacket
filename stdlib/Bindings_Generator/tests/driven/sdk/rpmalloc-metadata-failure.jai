// Public API behavior under actual native metadata allocation rejection.
RPM :: #import "rpmalloc";

probe :: #library,system "rpmalloc_metadata_fault";
metadata_fail_begin :: (ordinal: u32) #foreign probe;
metadata_fail_end :: () -> u32 #foreign probe;
metadata_worker :: (procedure: #type () #c_call) -> s32 #foreign probe;
metadata_fail_abort :: (line: u64) #foreign probe;

check :: (condition: bool, location: Source_Code_Location = #caller_location) {
    if !condition metadata_fail_abort(cast(u64) location.line_number);
}

handoff: *void;
mapped_before_free: s64;

worker :: () #c_call {
    push_context, defer_pop;
    metadata_fail_begin(1);
    RPM.thread_initialize();
    denied := metadata_fail_end();

    check(denied == 1 && !RPM.is_thread_initialized());
    before: RPM.rpmalloc_global_statistics_t;

    RPM.global_statistics(*before);
    mapped_before_free = before.mapped;
    check((cast(*u8) handoff)[0] == 91);
    metadata_fail_begin(1);
    RPM.rpfree(handoff);
    denied = metadata_fail_end();
    check(denied == 0);
    RPM.thread_initialize();
    check(RPM.is_thread_initialized());
    RPM.thread_finalize(true);
    check(!RPM.is_thread_initialized());
}

main :: () {
    metadata_fail_begin(1);
    caps := RPM.rpmalloc_allocator_proc(.CAPS, 0, 0, null, null);
    denied_caps := metadata_fail_end();

    check(caps != null && denied_caps == 0 && !RPM.is_initialized());
    rejected: u32;

    for 1..10 {
        metadata_fail_begin(cast(u32) it);
        result := RPM.initialize();
        denied := metadata_fail_end();

        if denied {
            rejected += 1;
            check(result == 12 && !RPM.is_initialized());
        } else {
            check(result == 0 && RPM.is_initialized());
            RPM.finalize();
        }

        check(RPM.initialize() == 0 && RPM.is_initialized());
        RPM.finalize();
    }

    check(rejected > 0);
    check(RPM.initialize() == 0);
    metadata_fail_begin(1);
    failed := RPM.rpmalloc(1_048_576);
    denied := metadata_fail_end();

    check(failed == null && denied == 1);
    memory := RPM.rpmalloc(8_388_608);

    check(memory != null);
    (cast(*u8) memory)[0] = 71;
    (cast(*u8) memory)[8_388_607] = 74;
    metadata_fail_begin(1);
    failed = RPM.rprealloc(memory, 16_777_216);
    denied = metadata_fail_end();
    check(failed == null && denied == 1);
    check((cast(*u8) memory)[0] == 71 && (cast(*u8) memory)[8_388_607] == 74);
    RPM.rpfree(memory);
    handoff = RPM.rpmalloc(8_388_608);
    check(handoff != null);
    (cast(*u8) handoff)[0] = 91;
    check(metadata_worker(worker) == 0);
    // Foreign-thread frees may be deferred to the owning heap. Drain that
    // lifecycle before asserting native mapped accounting decreased.
    RPM.thread_finalize(true);
    after: RPM.rpmalloc_global_statistics_t;

    RPM.global_statistics(*after);
    check(after.mapped < mapped_before_free);
    RPM.finalize();
}
