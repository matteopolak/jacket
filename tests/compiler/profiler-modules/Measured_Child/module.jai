#import "Basic";
P :: #import "POSIX";

child_call :: () -> s64 {
    duration: P.timespec;
    duration.tv_nsec = 3_000_000;
    assert(P.nanosleep(*duration, null) == 0);
    return 42;
}
