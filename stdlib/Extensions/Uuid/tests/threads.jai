#import "Basic";
#import "Extensions/Uuid";
Counter_Uuid :: #import "Extensions/Uuid";
T :: #import "Thread";

Shared :: struct {
    state: u64;
}

Worker :: struct {
    shared: *Shared;
    values: [200] Uuid;
}

work :: (thread: *T.Thread) -> s64 {
    worker := cast(*Worker) thread.data;
    for *value: worker.values {
        id, error := Counter_Uuid.v7_advance(*worker.shared.state, 123456, UUID_NIL);
        assert(error.kind == .NONE);
        value.* = id;
    }

    return 0;
}

concurrent_counter :: () {
    shared: Shared;
    workers: [4] Worker;
    threads: [4] T.Thread;

    for i: 0..3 {
        workers[i].shared = *shared;
        assert(T.thread_init(*threads[i], work));
        threads[i].data = *workers[i];
        T.thread_start(*threads[i]);
    }

    for *thread: threads {
        assert(T.thread_is_done(thread, -1));
        T.thread_deinit(thread);
    }

    seen: [801] bool;

    for worker: workers {
        previous := UUID_NIL;

        for value: worker.values {
            assert(uuid_compare(previous, value) < 0);
            counter := ((cast(int) value.bytes[6] & 15) << 8) | value.bytes[7];

            assert(counter >= 1 && counter <= 800 && !seen[counter]);
            seen[counter] = true;
            previous = value;
        }
    }

    for i: 1..800 assert(seen[i]);
} @test
