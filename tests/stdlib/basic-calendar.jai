// Calendar conversion: UTC exactness, round trips, normalization, timespec.
#import "Basic";

check_utc :: (year: s32, month0: s8, day0: s8, dow0: s8, h: s8, m: s8, s: s8, ms: s16) -> bool {
    ct := Calendar_Time.{year = year, month_starting_at_0 = month0, day_of_month_starting_at_0 = day0,
        day_of_week_starting_at_0 = dow0, hour = h, minute = m, second = s, millisecond = ms, time_zone = .UTC};
    at := calendar_to_apollo(ct);
    back := to_calendar(at, .UTC);
    return back.year == ct.year && back.month_starting_at_0 == month0 && back.day_of_month_starting_at_0 == day0
        && back.day_of_week_starting_at_0 == dow0 && back.hour == h && back.minute == m && back.second == s && back.millisecond == ms;
}

main :: () -> s64 {
    // Unix epoch.
    epoch := to_calendar(APOLLO_TIME_FOR_JAN_1_1970);
    if epoch.year != 1970 || epoch.month_starting_at_0 != 0 || epoch.day_of_month_starting_at_0 != 0 || epoch.day_of_week_starting_at_0 != 4 return 1;
    // Apollo 11 landing: Sunday 1969-07-20 20:17:39 UTC (Apollo_Time zero).
    moon := to_calendar(Apollo_Time.{});
    if calendar_to_iso_string(moon) != "1969-07-20T20:17:39.000Z" return 2;
    if moon.day_of_week_starting_at_0 != 0 return 3;
    if !check_utc(1969, 6, 19, 0, 20, 17, 39, 0) return 4;
    if !check_utc(2000, 1, 28, 2, 12, 0, 0, 0) return 5;      // 2000-02-29 Tuesday (leap, divisible by 400)
    if !check_utc(2024, 11, 30, 2, 23, 59, 59, 999) return 6;  // 2024-12-31 Tuesday
    if !check_utc(1900, 2, 0, 4, 0, 0, 1, 1) return 7;         // 1900-03-01 Thursday (1900 not leap)
    if !check_utc(1600, 0, 0, 6, 1, 2, 3, 4) return 8;         // 1600-01-01 Saturday
    if !check_utc(2100, 1, 27, 0, 5, 6, 7, 8) return 9;        // 2100-02-28 Sunday
    // Known unix timestamp: 1234567890 == 2009-02-13 23:31:30 UTC, Friday.
    stamp := APOLLO_TIME_FOR_JAN_1_1970 + seconds_to_apollo(1234567890);
    ts := to_calendar(stamp);
    if ts.year != 2009 || ts.month_starting_at_0 != 1 || ts.day_of_month_starting_at_0 != 12 || ts.hour != 23 || ts.minute != 31 || ts.second != 30 || ts.day_of_week_starting_at_0 != 5 return 10;
    if calendar_to_apollo(ts) != stamp return 11;
    // Normalization of out-of-range fields: month 12 of 2023 == January 2024.
    norm := calendar_to_apollo(Calendar_Time.{year = 2023, month_starting_at_0 = 12, day_of_month_starting_at_0 = 0, time_zone = .UTC});
    n := to_calendar(norm);
    if n.year != 2024 || n.month_starting_at_0 != 0 || n.day_of_month_starting_at_0 != 0 return 12;
    // Sweep: every 37 hours and 13 seconds across ~140 years round-trips exactly.
    t := APOLLO_TIME_FOR_JAN_1_1970 - seconds_to_apollo(2_000_000_000);
    end := APOLLO_TIME_FOR_JAN_1_1970 + seconds_to_apollo(2_500_000_000);
    step := seconds_to_apollo(37 * 3600 + 13);
    while t < end {
        if calendar_to_apollo(to_calendar(t)) != t return 13;
        t = t + step;
    }
    // Local zone round trip (offset is system dependent, but conversions must invert each other).
    now := seconds_to_apollo(1700000000) + APOLLO_TIME_FOR_JAN_1_1970;
    lc := to_calendar(now, .LOCAL);
    if lc.time_zone != .LOCAL || calendar_to_apollo(lc) != now return 14;
    #if OS != .WINDOWS {
        spec, spec_ok := to_timespec(APOLLO_TIME_FOR_JAN_1_1970 + seconds_to_apollo(5) + nanoseconds_to_apollo(250));
        if !spec_ok || spec.tv_sec != 5 || spec.tv_nsec != 250 return 15;
        negative, negative_ok := to_timespec(APOLLO_TIME_FOR_JAN_1_1970 - seconds_to_apollo(1));
        if negative_ok return 16;
    }
    print("ok\n");
    return 0;
}
