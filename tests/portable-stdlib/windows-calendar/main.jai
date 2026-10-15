#import "Basic";

main :: () {
    #if OS == .WINDOWS {
        epoch := to_calendar(APOLLO_TIME_FOR_JAN_1_1970);

        assert(epoch.year == 1970 && epoch.month_starting_at_0 == 0 && epoch.day_of_month_starting_at_0 == 0);
        assert(epoch.day_of_week_starting_at_0 == 4);
        assert(compare_apollo_times(calendar_to_apollo(epoch), APOLLO_TIME_FOR_JAN_1_1970) == 0);
        leap := epoch;
        leap.year = 2000;
        leap.month_starting_at_0 = 1;
        leap.day_of_month_starting_at_0 = 28;
        leap.hour = 12;
        leap.minute = 34;
        leap.second = 56;
        leap.millisecond = 789;
        at := calendar_to_apollo(leap);

        assert(compare_apollo_times(at, APOLLO_TIME_INVALID) != 0);
        result := to_calendar(at);

        assert(result.year == 2000 && result.month_starting_at_0 == 1 && result.day_of_month_starting_at_0 == 28);
        assert(result.hour == 12 && result.minute == 34 && result.second == 56 && result.millisecond == 789);
        leap.year = 1900;
        assert(compare_apollo_times(calendar_to_apollo(leap), APOLLO_TIME_INVALID) == 0);
        leap.year = 1599;
        assert(compare_apollo_times(calendar_to_apollo(leap), APOLLO_TIME_INVALID) == 0);

        for zone: Time_Zone.[.UTC, .LOCAL] {
            now := current_time_consensus();
            local := to_calendar(now, zone);
            rounded := calendar_to_apollo(local);

            assert(compare_apollo_times(rounded, APOLLO_TIME_INVALID) != 0);
            back := to_calendar(rounded, zone);

            assert(back.year == local.year && back.month_starting_at_0 == local.month_starting_at_0 && back.day_of_month_starting_at_0 == local.day_of_month_starting_at_0);
            assert(back.hour == local.hour && back.minute == local.minute && back.second == local.second && back.millisecond == local.millisecond);

            if zone == .UTC {
                original_ms, valid := to_milliseconds(now);
                rounded_ms, rounded_valid := to_milliseconds(rounded);
                assert(valid && rounded_valid && original_ms == rounded_ms);
            }
        }

        print("ok\n");
    } else {
        assert(false, "Windows calendar execution requires an actual Windows host");
    }
}
