#scope_module
#import "Basic";

#if OS == .WINDOWS Profile_Windows :: #import "Windows";
else #if OS == .MACOS || OS == .LINUX || OS == .ANDROID Profile_POSIX :: #import "POSIX";

profile_clock_frequency :: () -> s64, bool {
    #if OS == .WINDOWS {
        frequency: Profile_Windows.LARGE_INTEGER;
        if !Profile_Windows.QueryPerformanceFrequency(*frequency) || frequency.QuadPart <= 0 {
            log_error("Iprof: native performance-counter frequency unavailable\n");
            return 0, false;
        }

        return frequency.QuadPart, true;
    } else #if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
        return 1_000_000_000, true;
    } else {
        log_error("Iprof: no native profiling clock for this target\n");
        return 0, false;
    }
}

profile_clock_ticks :: () -> s64, bool {
    #if OS == .WINDOWS {
        count: Profile_Windows.LARGE_INTEGER;
        if !Profile_Windows.QueryPerformanceCounter(*count) || count.QuadPart < 0 {
            log_error("Iprof: native performance-counter read failed\n");
            return 0, false;
        }

        return count.QuadPart, true;
    } else #if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
        time: Profile_POSIX.timespec;
        if Profile_POSIX.clock_gettime(.MONOTONIC, *time) != 0 {
            log_error("Iprof: native monotonic clock read failed\n");
            return 0, false;
        }

        maximum :: 0x7fffffffffffffff;
        frequency :: 1_000_000_000;

        if time.tv_sec < 0 || time.tv_nsec < 0 || time.tv_nsec >= frequency || time.tv_sec > maximum / frequency || (time.tv_sec == maximum / frequency && time.tv_nsec > maximum % frequency) {
            log_error("Iprof: native clock value is not representable\n");
            return 0, false;
        }

        return time.tv_sec * frequency + time.tv_nsec, true;
    } else {
        log_error("Iprof: no native profiling clock for this target\n");
        return 0, false;
    }
}
