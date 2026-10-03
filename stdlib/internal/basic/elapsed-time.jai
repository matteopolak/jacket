#if OS == .MACOS || OS == .LINUX || OS == .ANDROID || OS == .WINDOWS {
    seconds_since_init :: () -> float64 {
        initializing, state := compare_and_swap(*elapsed_state, cast(s32) 0, cast(s32) 1);
        if initializing {
            origin := current_time_monotonic();

            elapsed_origin = origin;
            ready: s32 = 2;
            if compare_apollo_times(origin, APOLLO_TIME_INVALID) == 0 ready = 3;

            published, previous := compare_and_swap(*elapsed_state, cast(s32) 1, ready);

            assert(published && previous == 1, "elapsed-time initialization state corrupted");
            if ready == 3 return time_failure_nan("elapsed-time origin query failed");

            return 0;
        }

        while state != 2 {
            if state == 3 return time_failure_nan("elapsed-time origin is unavailable");
            if context.handling_assertion_failure return time_quiet_nan();
            assert(state == 1, "elapsed-time initialization state corrupted");
            initializing, state = compare_and_swap(*elapsed_state, cast(s32) 2, cast(s32) 2);
        }

        now := current_time_monotonic();
        if compare_apollo_times(now, APOLLO_TIME_INVALID) == 0 return time_failure_nan("elapsed-time clock query failed");
        if compare_apollo_times(now, elapsed_origin) < 0 return time_failure_nan("elapsed-time monotonic clock moved backwards");

        seconds, valid := to_float64_seconds(elapsed_difference(now, elapsed_origin));

        if !valid return time_failure_nan("elapsed-time duration conversion failed");

        return seconds;
    }
}

#scope_file
elapsed_state: s32;
elapsed_origin: Apollo_Time;

elapsed_difference :: (now: Apollo_Time, origin: Apollo_Time) -> Apollo_Time #no_aoc {
    low := now.low - origin.low;
    high := cast(u64) now.high - cast(u64) origin.high;
    if now.low < origin.low high -= 1;
    return .{ low, cast(s64) high };
}
