#import "Basic";
#import "Thread";

State :: struct {
    mutex: Mutex;
    count: int;
    closed: bool;
}

// A path that returns while the lock is held.
bump :: (state: *State) -> bool {
    lock(*state.mutex);
    if state.closed return false;

    state.count += 1;
    unlock(*state.mutex);

    return true;
}

// The end of the body is reached with the lock held.
forget :: (state: *State) {
    lock(*state.mutex);
    state.count = 0;
}

main :: () {
    state: State;
    init(*state.mutex);
    bump(*state);
    forget(*state);
}
