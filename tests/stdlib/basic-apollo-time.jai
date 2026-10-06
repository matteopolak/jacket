// Scalar time boundary behavior plus the declared legacy Calendar_Time schema.
#import "Basic";

main :: () -> s64 {
    minimum: s64 = -9223372036854775808;
    converted := seconds_to_apollo(minimum);
    back, back_ok := to_seconds(converted);
    if !back_ok || back != minimum return 1;
    partial := milliseconds_to_apollo(-1500);
    truncated, truncated_ok := to_seconds(partial);
    rounded, rounded_ok := to_seconds(partial, DO_ROUNDING = true);
    if !truncated_ok || truncated != -1 || !rounded_ok || rounded != -2 return 2;
    positive, positive_ok := to_seconds(milliseconds_to_apollo(1500), DO_ROUNDING = true);
    if !positive_ok || positive != 2 return 3;
    invalid, invalid_ok := to_nanoseconds(APOLLO_TIME_INVALID);
    if invalid_ok return 4;
    out_of_range, range_ok := to_nanoseconds(seconds_to_apollo(9223372036854775807));
    if range_ok return 5;
    float_time := to_apollo(1.5);
    millis, millis_ok := to_milliseconds(float_time);
    if !millis_ok || millis != 1500 || to_float64_seconds(float_time) != 1.5 return 6;
    calendar := Calendar_Time.{ year = 2026, month_starting_at_0 = 9, day_of_month_starting_at_0 = 1,
        day_of_week_starting_at_0 = 5, hour = 3, minute = 4, second = 5, millisecond = 6, time_zone = .UTC };
    if calendar_to_string(calendar) != "2026-10-02 03:04:05.006" return 7;
    if calendar_to_iso_string(calendar) != "2026-10-02T03:04:05.006Z" return 8;
    return 0;
}
