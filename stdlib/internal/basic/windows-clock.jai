#if OS == .WINDOWS {
    current_time_consensus :: () -> Apollo_Time {
        now: Clock_Windows.FILETIME;
        Clock_Windows.GetSystemTimeAsFileTime(*now);
        return filetime_to_apollo(now);
    }

    current_time_monotonic :: () -> Apollo_Time {
        #if OS != BUILD_OS {
            if #compile_time return host_monotonic_time();
        }

        frequency := get_counter_frequency();
        if frequency <= 0 return APOLLO_TIME_INVALID;

        counter: Clock_Windows.LARGE_INTEGER;
        status := Clock_Windows.QueryPerformanceCounter(*counter);

        if status == 0 || counter.QuadPart < 0 {
            report_time_failure("Windows monotonic clock query failed or returned a negative counter");
            return APOLLO_TIME_INVALID;
        }

        low, high, valid := Counter_Kernel.counter_to_words(counter.QuadPart, frequency);

        if !valid {
            report_time_failure("Windows monotonic counter conversion failed");
            return APOLLO_TIME_INVALID;
        }

        return .{ low, high };
    }
}

#scope_file
#if OS == .WINDOWS {
    Clock_Windows :: #import "Windows";
    Counter_Kernel :: #import "internal/time/counter";

    counter_frequency: s64;

    get_counter_frequency :: () -> s64 {
        uninitialized, value := compare_and_swap(*counter_frequency, cast(s64) 0, cast(s64) 0);
        if !uninitialized return value;

        frequency: Clock_Windows.LARGE_INTEGER;

        status := Clock_Windows.QueryPerformanceFrequency(*frequency);

        if status == 0 || frequency.QuadPart <= 0 {
            report_time_failure("Windows monotonic clock frequency query failed");
            return 0;
        }

        inserted, existing := compare_and_swap(*counter_frequency, cast(s64) 0, frequency.QuadPart);
        if inserted return frequency.QuadPart;

        return existing;
    }
}
