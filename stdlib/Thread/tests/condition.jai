#import "Basic";
T :: #import "Thread";

State :: struct {
    mutex: T.Mutex;
    cv: T.Condition_Variable;
    ready: T.Semaphore;
    go: bool;
    result: s64;
}

worker :: (thread: *T.Thread) -> s64 {
    state := cast(*State) thread.data;

    assert(thread.starting_temporary_storage.size == 64);
    assert(talloc(8) != null);
    T.lock(*state.mutex);
    T.signal(*state.ready);
    while !state.go T.wait(*state.cv, *state.mutex);

    state.result = 17;
    T.unlock(*state.mutex);

    return 0;
}

main :: () {
    state: State;

    T.init(*state.mutex);
    T.init(*state.cv);
    T.init(*state.ready);
    T.lock(*state.mutex);
    T.lock(*state.mutex);
    T.unlock(*state.mutex);
    T.unlock(*state.mutex);
    buffer: [64] u8;
    storage: Temporary_Storage;
    storage.data = buffer.data;
    storage.size = buffer.count;
    thread: T.Thread;
    thread.data = *state;
    assert(T.thread_init(*thread, worker, starting_storage = *storage));
    storage.size = 0;
    T.thread_start(*thread);
    assert(T.wait_for(*state.ready, 5000) == .SUCCESS);
    T.lock(*state.mutex);
    state.go = true;
    T.wake(*state.cv);
    T.unlock(*state.mutex);
    assert(T.thread_is_done(*thread, 5000));
    T.thread_deinit(*thread);
    assert(state.result == 17);
    T.destroy(*state.ready);
    T.destroy(*state.cv);
    T.destroy(*state.mutex);
    print("ok\n");
}
