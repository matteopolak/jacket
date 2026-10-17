#import "Basic";
#if OS == .WINDOWS W :: #import "Windows";
else P :: #import "POSIX";

main :: () {
    before := monotonic_seconds();

    sleep_milliseconds(20);
    elapsed := monotonic_seconds() - before;

    assert(elapsed >= 0.019);
    sleep_milliseconds(0);
    print("ok\n");
}

monotonic_seconds :: () -> float64 {
    #if OS == .WINDOWS {
        counter, frequency: s64;
        assert(W.QueryPerformanceCounter(*counter) != 0);
        assert(W.QueryPerformanceFrequency(*frequency) != 0 && frequency > 0);
        return cast(float64) counter / cast(float64) frequency;
    } else {
        value: P.timespec;
        assert(P.clock_gettime(P.clockid_t.MONOTONIC, *value) == 0);
        return cast(float64) value.tv_sec + cast(float64) value.tv_nsec / 1000000000.0;
    }
}
