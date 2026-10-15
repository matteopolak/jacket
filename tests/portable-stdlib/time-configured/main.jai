#import "Basic";
Configured :: #import "Basic"(ENABLE_ASSERT = false);

main :: () {
    assert(type_info(Configured.Apollo_Time).name == "Apollo_Time");
    half := Configured.to_apollo(0.5);
    expected := Configured.milliseconds_to_apollo(500);

    assert(Configured.compare_apollo_times(half, expected) == 0);
    sample := Configured.Calendar_Time.{ year = 2000, month_starting_at_0 = 1, day_of_month_starting_at_0 = 28 };

    assert(Configured.calendar_to_string(sample) == "29 February 2000, 00:00:00");
    assert(Configured.calendar_to_iso_string(sample) == "2000-02-29T00:00:00.000Z");
    first := Configured.seconds_since_init();

    assert(first == 0);
    assert(Configured.seconds_since_init() >= 0);

    #if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
        epoch := Configured.timespec_to_apollo(.{ 0, 0 });

        assert(Configured.compare_apollo_times(epoch, Configured.APOLLO_TIME_FOR_JAN_1_1970) == 0);
        calendar := Configured.to_calendar(epoch);

        assert(calendar.year == 1970 && calendar.month_starting_at_0 == 0 && calendar.day_of_month_starting_at_0 == 0);
        assert(Configured.compare_apollo_times(Configured.calendar_to_apollo(calendar), epoch) == 0);
        configured_now := Configured.current_time_consensus();
        configured_seconds, configured_valid := Configured.to_float64_seconds(configured_now);
        reference_seconds, reference_valid := to_float64_seconds(current_time_consensus());

        assert(configured_valid && reference_valid && configured_seconds - reference_seconds >= -2 && configured_seconds - reference_seconds <= 2);
        pause := P.timespec.{ 0, 30000000 };
        remaining: P.timespec;

        while P.nanosleep(*pause, *remaining) != 0 {
            assert(P.errno() == P.EINTR);
            pause = remaining;
        }

        assert(Configured.seconds_since_init() >= 0.01);
        previous := context.logger;

        context.logger = capture;
        invalid := Configured.to_calendar(Configured.APOLLO_TIME_INVALID);

        context.logger = previous;
        assert(reported == 1 && invalid.year == 0 && invalid.month_starting_at_0 == 0 && invalid.day_of_month_starting_at_0 == 0);
        assert(!context.handling_assertion_failure);
    }

    #if OS == .WINDOWS {
        zero: W.FILETIME;
        at := Configured.filetime_to_apollo(zero);
        ft, valid := Configured.to_filetime(at);

        assert(valid && ft.dwLowDateTime == 0 && ft.dwHighDateTime == 0);
        start := Configured.current_time_monotonic();
        finish := Configured.current_time_monotonic();

        assert(Configured.compare_apollo_times(finish, start) >= 0);
        now := Configured.current_time_consensus();

        ft, valid = Configured.to_filetime(now);
        assert(valid);
        epoch_calendar := Configured.to_calendar(Configured.APOLLO_TIME_FOR_JAN_1_1970);

        assert(epoch_calendar.year == 1970 && epoch_calendar.day_of_week_starting_at_0 == 4);
        assert(Configured.compare_apollo_times(Configured.calendar_to_apollo(epoch_calendar), Configured.APOLLO_TIME_FOR_JAN_1_1970) == 0);
    }

    print("ok\n");
}

#scope_file
reported: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    reported += 1;
    assert(message.count > 0 && (info.common_flags & .ERROR) != 0);
}

#if OS == .WINDOWS W :: #import "Windows";
#if OS == .MACOS || OS == .LINUX || OS == .ANDROID P :: #import "POSIX";
