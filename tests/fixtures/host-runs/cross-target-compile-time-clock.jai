// Compile-time code runs on the compiler's host whatever target is selected, so a plain `#run`
// reads the host's monotonic clock (the target's clock identifiers mean something else there).
#import "Basic";

#run {
    first := current_time_monotonic();
    second := current_time_monotonic();
    assert(first != APOLLO_TIME_INVALID && second != APOLLO_TIME_INVALID, "the clock query failed");
    assert(compare_apollo_times(second, first) >= 0, "the clock went backwards");
}

main :: () {}
