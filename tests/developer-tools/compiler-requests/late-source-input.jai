#import "Basic";

original_target :: () -> s64 {
    return 0;
}

main :: () {
    assert(original_target() == 42);
    print("ok\n");
}
