#import "Basic";
P :: #import "POSIX";

main :: () {
    before: P.timespec;
    after: P.timespec;

    assert(P.clock_gettime(P.clockid_t.MONOTONIC, *before) == 0);
    sleep_milliseconds(20);
    assert(P.clock_gettime(P.clockid_t.MONOTONIC, *after) == 0);
    elapsed := cast(float64) (after.tv_sec - before.tv_sec) + cast(float64) (after.tv_nsec - before.tv_nsec) / 1000000000.0;

    assert(elapsed >= 0.019);
    sleep_milliseconds(0);
    print("ok\n");
}
