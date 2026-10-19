// Basic's clock, calendar, sleep and exit on the sandbox host (`-os wasm`, the browser's engine).
#import "Basic";

main :: () {
    start := current_time_monotonic();

    sleep_milliseconds(1500);
    elapsed := to_float64_seconds(current_time_monotonic() - start);

    leap_day: Calendar_Time;
    leap_day.year = 2024;
    leap_day.month_starting_at_0 = 1;
    leap_day.day_of_month_starting_at_0 = 28;
    leap_day.hour = 12;

    print("% % %\n", elapsed >= 1.5, seconds_since_init() >= 0, to_calendar(current_time_consensus()).year >= 2023);
    print("%\n", calendar_to_string(to_calendar(calendar_to_apollo(leap_day))));
    exit(3);
}
