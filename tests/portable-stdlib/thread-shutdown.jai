#import "Basic";
T :: #import "Thread";

State :: struct {
    entered: T.Semaphore;
    gate: T.Semaphore;
    result: s64;
}

work :: (group: *T.Thread_Group, thread: *T.Thread, data: *void) -> T.Thread_Continue_Status {
    state := cast(*State) data;

    T.signal(*state.entered);
    assert(T.wait_for(*state.gate, -1) == .SUCCESS);
    state.result = 1;

    return .CONTINUE;
}

main :: () {
    state: State;

    T.init(*state.entered);
    T.init(*state.gate);
    group: T.Thread_Group;
    group.logging = false;
    T.init(*group, 1, work);
    T.add_work(*group, *state);
    T.start(*group);
    assert(T.wait_for(*state.entered, 5000) == .SUCCESS);
    assert(!T.shutdown(*group, 0));
    assert(group.initted && group.worker_info.count == 1);
    T.signal(*state.gate);
    assert(T.shutdown(*group, 5000));
    assert(state.result == 1);
    T.destroy(*state.entered);
    T.destroy(*state.gate);
    print("ok\n");
}
