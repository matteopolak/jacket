#import "Basic";
Again :: #import "Basic";

main :: () {
    #if OS == .MACOS || OS == .LINUX || OS == .ANDROID || OS == .WINDOWS {
        pause_before_first_call();
        first := seconds_since_init();

        assert(first == 0, "elapsed-time origin must be the first call");
        before := current_time_monotonic();

        pause_before_first_call();
        elapsed := Again.seconds_since_init();

        assert(elapsed >= 0.01 && elapsed < 10);
        previous := elapsed;

        for i: 0..255 {
            current := seconds_since_init();
            assert(current >= previous);
            previous = current;
        }

        assert(compare_apollo_times(current_time_monotonic(), before) >= 0);
    }

    print("ok\n");
}

#scope_file
#if OS == .WINDOWS {
    W :: #import "Windows";

    pause_before_first_call :: () {
        W.Sleep(30);
    }
} else #if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
    P :: #import "POSIX";

    pause_before_first_call :: () {
        request := P.timespec.{ 0, 30000000 };
        remaining: P.timespec;

        while P.nanosleep(*request, *remaining) != 0 {
            assert(P.errno() == P.EINTR, "elapsed fixture sleep failed");
            request = remaining;
        }
    }
}
