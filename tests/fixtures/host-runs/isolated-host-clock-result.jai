// Explicit no-argument primitive result, matching the reviewed isolated host domain.
HOST_RESULT :: #run,host -> bool {
    B :: #import "Basic";
    first := B.current_time_monotonic();
    second := B.current_time_monotonic();
    return B.compare_apollo_times(second, first) >= 0;
};

#assert HOST_RESULT;
main :: () {}
