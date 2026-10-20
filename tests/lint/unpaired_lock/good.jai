#import "Basic";
#import "Thread";

State :: struct {
    mutex: Mutex;
    other: Mutex;
    count: int;
    closed: bool;
}

// Released by `defer` right after locking.
deferred :: (state: *State) -> bool {
    lock(*state.mutex);
    defer unlock(*state.mutex);

    if state.closed return false;

    state.count += 1;

    return true;
}

// Released before each return.
explicit :: (state: *State) -> bool {
    lock(*state.mutex);

    if state.closed {
        unlock(*state.mutex);
        return false;
    }

    state.count += 1;
    unlock(*state.mutex);

    return true;
}

// Two locks, each released on its own expression.
both :: (state: *State) {
    lock(*state.mutex);
    lock(*state.other);
    state.count += 1;
    unlock(*state.other);
    unlock(*state.mutex);
}

// A wrapper: the caller unlocks.
acquire :: (state: *State) {
    lock(*state.mutex);
} @Locks

release :: (state: *State) {
    unlock(*state.mutex);
} @Unlocks

// Loops and branches the scan cannot follow stay quiet.
looping :: (state: *State, rounds: int) {
    lock(*state.mutex);

    for 1..rounds {
        if state.closed break;
        state.count += 1;
    }

    unlock(*state.mutex);
}

main :: () {
    state: State;

    init(*state.mutex);
    init(*state.other);
    deferred(*state);
    explicit(*state);
    both(*state);
    acquire(*state);
    release(*state);
    looping(*state, 2);
}
