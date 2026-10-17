#import "Basic";

counter: s64;

enter_hook :: () {
    counter += 1;
}

leave_hook :: () {
    counter += 100;
}

original_target :: (value: s64) -> s64 {
    return value + 1;
}

main :: () {
    assert(original_target(41) == 42);
    assert(counter == 101);
    print("ok\n");
}
