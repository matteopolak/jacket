// Reviewed basic.md: POSIX normalization and -1 sentinel collision.
#import "Basic";

main :: () {
    #if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
        cases := Calendar_Time.[
            .{ year = 2000, month_starting_at_0 = 12, day_of_month_starting_at_0 = 0 },
            .{ year = 2000, month_starting_at_0 = -1, day_of_month_starting_at_0 = 31 },
            .{ year = 2100, month_starting_at_0 = 1, day_of_month_starting_at_0 = 28 },
            .{ year = 2000, hour = 30, minute = -1, second = 60, millisecond = 1500 },
            .{ year = 2000, day_of_month_starting_at_0 = -1, millisecond = -500 }
        ];

        for zone: Time_Zone.[.UTC, .LOCAL] {
            for input: cases {
                ct := input;
                ct.time_zone = zone;
                broken: P.tm;
                broken.tm_year = ct.year - 1900;
                broken.tm_mon = ct.month_starting_at_0;
                broken.tm_mday = cast(s32) ct.day_of_month_starting_at_0 + 1;
                broken.tm_hour = ct.hour;
                broken.tm_min = ct.minute;
                broken.tm_sec = ct.second;
                broken.tm_isdst = -1;
                seconds: P.time_t;
                if zone == .UTC seconds = P.timegm(*broken);
                else seconds = P.mktime(*broken);

                assert(seconds != -1);
                extra_seconds := cast(s64) ct.millisecond / 1000;
                nanos := (cast(s64) ct.millisecond % 1000) * 1000000;

                if nanos < 0 {
                    extra_seconds -= 1;
                    nanos += 1000000000;
                }

                expected := timespec_to_apollo(.{ seconds + extra_seconds, nanos });
                actual := calendar_to_apollo(ct);

                assert(compare_apollo_times(actual, expected) == 0);
                ct.day_of_week_starting_at_0 = 127;
                assert(compare_apollo_times(calendar_to_apollo(ct), expected) == 0);
            }
        }

        before := to_calendar(timespec_to_apollo(.{ -1, 0 }));
        collision := calendar_to_apollo(before);

        assert(collision.low == 0 && collision.high == 0);
    }

    print("ok\n");
}

#scope_file
#if OS == .MACOS || OS == .LINUX || OS == .ANDROID P :: #import "POSIX";
