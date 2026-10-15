#import "Basic";

main :: () {
    for value: s64.[-1234567, -2, -1, 0, 1, 2, 1234567] {
        actual := to_apollo(cast(float64) value);
        expected := seconds_to_apollo(value);
        assert(actual.low == expected.low && actual.high == expected.high);
    }

    half := to_apollo(0.5);

    assert(half.low == 500000000000000 && half.high == 0);
    negative := to_apollo(-0.5);
    expected := milliseconds_to_apollo(-500);

    assert(negative.low == expected.low && negative.high == expected.high);
    huge := to_apollo(18446744073709551616.0);

    assert(huge.low == 0 && huge.high == FEMTOSECONDS);
    negative_zero := to_apollo(-0.0);

    assert(negative_zero.low == 0 && negative_zero.high == 0);
    tiny := to_apollo(1e-300);

    assert(tiny.low == 0 && tiny.high == 0);
    overflow := to_apollo(1e300);

    assert(overflow.low == APOLLO_TIME_INVALID.low && overflow.high == APOLLO_TIME_INVALID.high);
    bits: u64 = 0x7ff0000000000000;
    infinite: float64;

    memcpy(*infinite, *bits, 8);
    invalid := to_apollo(infinite);

    assert(invalid.low == APOLLO_TIME_INVALID.low && invalid.high == APOLLO_TIME_INVALID.high);
    bits = 0x7ff8000000000001;
    memcpy(*infinite, *bits, 8);
    invalid = to_apollo(infinite);
    assert(invalid.low == APOLLO_TIME_INVALID.low && invalid.high == APOLLO_TIME_INVALID.high);

    #if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
        epoch := timespec_to_apollo(.{ 0, 0 });

        assert(epoch.low == APOLLO_TIME_FOR_JAN_1_1970.low && epoch.high == APOLLO_TIME_FOR_JAN_1_1970.high);

        for seconds: s64.[-2208988800, -1, 0, 1, 2147483648, 4107542400] {
            at := timespec_to_apollo(.{ seconds, 123456789 });
            ts, valid := to_timespec(at);
            assert(valid && ts.tv_sec == seconds && ts.tv_nsec == 123456789);
        }

        before := epoch;
        before.low -= 1;
        ts, valid := to_timespec(before);

        assert(valid && ts.tv_sec == -1 && ts.tv_nsec == 999999999);
        ts, valid = to_timespec(APOLLO_TIME_INVALID);
        assert(!valid);
        bad := timespec_to_apollo(.{ 0, 1000000000 });

        assert(compare_apollo_times(bad, APOLLO_TIME_INVALID) == 0);
        calendar := to_calendar(epoch);

        assert(calendar.year == 1970 && calendar.month_starting_at_0 == 0 && calendar.day_of_month_starting_at_0 == 0);
        assert(calendar.day_of_week_starting_at_0 == 4 && calendar.hour == 0 && calendar.minute == 0 && calendar.second == 0);
        assert(calendar.time_zone == .UTC && compare_apollo_times(calendar_to_apollo(calendar), epoch) == 0);
        calendar = .{ year = 2000, month_starting_at_0 = 1, day_of_month_starting_at_0 = 28,
                      hour = 12, minute = 34, second = 56, millisecond = 789, time_zone = .UTC };
        leap := calendar_to_apollo(calendar);
        roundtrip := to_calendar(leap);

        assert(roundtrip.year == 2000 && roundtrip.month_starting_at_0 == 1 && roundtrip.day_of_month_starting_at_0 == 28);
        assert(roundtrip.day_of_week_starting_at_0 == 2 && roundtrip.hour == 12 && roundtrip.minute == 34 && roundtrip.second == 56 && roundtrip.millisecond == 789);
        calendar.year = 2100;
        assert(compare_apollo_times(calendar_to_apollo(calendar), APOLLO_TIME_INVALID) == 0);
        prior := timespec_to_apollo(.{ -1, 0 });

        assert(compare_apollo_times(calendar_to_apollo(to_calendar(prior)), prior) == 0);
        local := to_calendar(leap, .LOCAL);

        assert(local.time_zone == .LOCAL && compare_apollo_times(calendar_to_apollo(local), leap) == 0);
        start := current_time_monotonic();
        finish := current_time_monotonic();

        assert(compare_apollo_times(finish, start) >= 0);
        now := current_time_consensus();
        now_ts, now_valid := to_timespec(now);

        assert(now_valid);
        native_now := Native_Time.time(null);

        assert(now_ts.tv_sec <= native_now && native_now - now_ts.tv_sec <= 1);
    }

    print("ok\n");
}

#scope_file
#if OS == .MACOS || OS == .LINUX || OS == .ANDROID Native_Time :: #import "POSIX";
