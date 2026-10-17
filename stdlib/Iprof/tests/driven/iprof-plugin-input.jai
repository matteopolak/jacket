#import "Basic";
P :: #import "POSIX";

leaf :: (early: bool) -> s64 {
    duration: P.timespec;
    duration.tv_nsec = 3_000_000;
    assert(P.nanosleep(*duration, null) == 0);
    if early return 7;

    return 8;
}

main :: () {
    assert(leaf(true) == 7);
    assert(leaf(false) == 8);
    print("ok\n");
}
