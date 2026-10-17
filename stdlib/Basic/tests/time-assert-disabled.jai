B :: #import "Basic";
Off :: #import "Basic"()(ENABLE_ASSERT = false);
P :: #import "POSIX";

main :: () {
    before: P.timespec;
    after: P.timespec;

    status := P.clock_gettime(.REALTIME, *before);

    B.assert(status == 0);
    consensus := Off.current_time_consensus();

    status = P.clock_gettime(.REALTIME, *after);
    B.assert(status == 0);
    actual, valid := Off.to_timespec(consensus);

    B.assert(valid);
    B.assert(actual.tv_sec > before.tv_sec || (actual.tv_sec == before.tv_sec && actual.tv_nsec >= before.tv_nsec));
    B.assert(actual.tv_sec < after.tv_sec || (actual.tv_sec == after.tv_sec && actual.tv_nsec <= after.tv_nsec));

    status = P.clock_gettime(.MONOTONIC, *before);
    B.assert(status == 0);
    monotonic := Off.current_time_monotonic();

    status = P.clock_gettime(.MONOTONIC, *after);
    B.assert(status == 0);
    lower := Off.seconds_to_apollo(before.tv_sec);
    upper := Off.seconds_to_apollo(after.tv_sec + 1);

    B.assert(Off.compare_apollo_times(monotonic, lower) >= 0);
    B.assert(Off.compare_apollo_times(monotonic, upper) < 0);
    B.print("ok\n");
}
