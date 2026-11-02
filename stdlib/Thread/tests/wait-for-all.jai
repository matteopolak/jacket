// test: skip_on(.WASM, reason = "needs the Thread module, which has no wasm target (no OS threads)")
#import "Basic";
#import "Atomics";
T :: #import "Thread";

JOBS :: 10000;
WORKERS :: 8;
ROUNDS :: 50;

#if OS == .WINDOWS {
    kernel32 :: #system_library "kernel32";
    set_error :: (code: u32) #foreign kernel32 "SetLastError";
}

Shared :: struct {
    ran: [JOBS] s64;
    started_mask: s64;
    starts: s64;
    gate: T.Semaphore;
}

shared: Shared;

tiny_job :: (group: *T.Thread_Group, thread: *T.Thread, work: *void) -> T.Thread_Continue_Status {
    slot := cast(*s64) work;

    slot.* += 1;

    return .CONTINUE;
}

gated_job :: (group: *T.Thread_Group, thread: *T.Thread, work: *void) -> T.Thread_Continue_Status {
    T.wait_for(*shared.gate, -1);

    return .CONTINUE;
}

on_start :: (group: *T.Thread_Group, thread: *T.Thread, worker_index: s32) {
    assert(thread.worker_info.worker_index == worker_index && context.thread_index == thread.index);
    atomic_or(*shared.started_mask, cast(s64) 1 << worker_index);
    atomic_increment(*shared.starts);
}

main :: () {
    #if OS == .WINDOWS {
        W :: #import "Windows";
        mutex: T.Mutex;
        changed: T.Condition_Variable;

        T.init(*mutex);
        T.init(*changed);
        T.lock(*mutex);
        set_error(1234);
        // An immediate timeout must replace the previous foreign call's error even in --interp.
        T.cvariable_wait(*changed.condition_variable, *mutex.csection, 0);
        assert(W.GetLastError() == 1460);
        T.unlock(*mutex);
        T.destroy(*changed);
        T.destroy(*mutex);
    }

    // Many tiny jobs over contended workers, waited for again and again on one group.
    group: T.Thread_Group;

    group.logging = false;
    group.worker_start = on_start;
    T.init(*group, WORKERS, tiny_job, enable_work_stealing = true);
    T.start(*group);

    for round: 1..ROUNDS {
        for 0..JOBS - 1 T.add_work(*group, *shared.ran[it]);

        assert(T.wait_for_all(*group));

        for shared.ran assert(it == round, "round %: job % ran % times", round, it_index, it);

        assert(T.get_completed_work(*group).count == JOBS);
        assert(T.wait_for_all(*group), "an idle group waits for nothing");
        reset_temporary_storage();
    }

    assert(T.shutdown(*group, 5000));

    // Every worker ran the start hook once, with its own index.
    assert(shared.starts == WORKERS && shared.started_mask == (1 << WORKERS) - 1);

    // Fresh groups, so that creating and finishing a group is covered too.
    for *shared.ran it.* = 0;

    for round: 1..ROUNDS {
        fresh: T.Thread_Group;

        fresh.logging = false;
        T.init(*fresh, WORKERS, tiny_job);
        T.start(*fresh);
        for 0..999 T.add_work(*fresh, *shared.ran[it]);

        assert(T.wait_for_all(*fresh));
        for 0..999 assert(shared.ran[it] == round);

        assert(T.shutdown(*fresh, 5000));
    }

    // A timeout says so, and the wait still works once the jobs go through.
    T.init(*shared.gate, 0);

    blocked: T.Thread_Group;

    blocked.logging = false;
    T.init(*blocked, 2, gated_job);
    T.start(*blocked);
    for 0..3 T.add_work(*blocked, null);

    assert(!T.wait_for_all(*blocked, 30));
    for 0..3 T.signal(*shared.gate);

    assert(T.wait_for_all(*blocked, 5000));
    assert(T.shutdown(*blocked, 5000));
    T.destroy(*shared.gate);

    print("ok\n");
}
