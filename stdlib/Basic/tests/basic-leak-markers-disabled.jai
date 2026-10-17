// Without the memory debugger the leak markers discard their argument expression.
#import "Basic";

discarded: int;

side_effect :: () -> *void {
    discarded += 1;
    return *discarded;
}

main :: () {
    this_allocation_is_not_a_leak(side_effect());
    this_allocation_leaks_todo(side_effect());
    this_allocation_does_not_propagate_leak_ownership(side_effect());
    assert(discarded == 0);
    print("ok\n");
}
