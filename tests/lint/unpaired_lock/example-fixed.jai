#import "Basic";
#import "Thread";

State :: struct {
    mutex: Mutex;
    count: int;
    closed: bool;
}

bump :: (state: *State) -> bool {
    lock(*state.mutex);
    defer unlock(*state.mutex);

    if state.closed return false;

    state.count += 1;

    return true;
}

main :: () {
    state: State;
    init(*state.mutex);
    print("%\n", bump(*state));
}
