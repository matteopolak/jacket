// Dates, clock readings and civil calendar arithmetic.

// Validates Gregorian components without clamping. Example: `d, error := date(2024, 2, 29)`.
date :: (year: s64, month: s64, day: s64) -> Date, Time_Error {
    n, e := days_in_month(year, month);
    if e.kind != .NONE return .{}, e;
    if day < 1 || day > n return .{}, .{ .INVALID_VALUE, "invalid day", -1 };
    return .{ cast(s16) year, cast(s8) month, cast(s8) day }, .{};
}

// Validates a clock reading; leap seconds are rejected. Example: `t, error := time_of_day(23, 59, 59, 1)`.
time_of_day :: (hour: s64, minute: s64, second: s64, nanosecond: s64 = 0) -> Time_Of_Day, Time_Error {
    if hour < 0 || hour > 23 || minute < 0 || minute > 59 || second < 0 || second > 59 || nanosecond < 0 || nanosecond >= NS return .{}, .{ .INVALID_VALUE, "invalid clock component", -1 };
    return .{ cast(s8) hour, cast(s8) minute, cast(s8) second, cast(s32) nanosecond }, .{};
}

// Combines validated civil components. Example: `dt := date_time(day, clock)`.
date_time :: (day: Date, clock: Time_Of_Day) -> Date_Time {
    return .{ day, clock };
}

// Validates a civil date and clock reading together. Example: `dt, error := date_time(2024, 3, 10, 2, 30)`.
date_time :: (year: s64, month: s64, day: s64, hour: s64 = 0, minute: s64 = 0, second: s64 = 0, nanosecond: s64 = 0) -> Date_Time, Time_Error {
    d, e := date(year, month, day);
    if e.kind != .NONE return .{}, e;

    t, te := time_of_day(hour, minute, second, nanosecond);
    if te.kind != .NONE return .{}, te;

    return .{ d, t }, .{};
}

// Returns the Gregorian month length after validating year and month. Example: `n, error := days_in_month(2024, 2)`.
days_in_month :: (year: s64, month: s64) -> s64, Time_Error {
    if year < -9999 || year > 9999 return 0, .{ .OUT_OF_RANGE, "year outside civil range", -1 };
    if month < 1 || month > 12 return 0, .{ .INVALID_VALUE, "invalid month", -1 };
    return month_length(year, month), .{};
}

// Tests the Gregorian leap rule within the supported year range. Example: `leap := is_leap_year(2000)`.
is_leap_year :: (year: s64) -> bool {
    return year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
}

// Returns the ISO weekday after validating the date. Example: `day := weekday(value)`.
weekday :: (value: Date) -> Weekday {
    return cast(Weekday) (modulo(date_days(value) + 3, 7) + 1);
}

// Returns the ordinal day, starting at one. Example: `ordinal := day_of_year(value)`.
day_of_year :: (value: Date) -> s64 {
    return date_days(value) - date_days(.{ value.year, 1, 1 }) + 1;
}

// Returns ISO week-year and week number; week one contains January 4. Example: `year, week := iso_week(value)`.
iso_week :: (value: Date) -> s64, s64 {
    thursday := date_days(value) + 4 - cast(s64) weekday(value);
    d := civil_day(thursday);
    jan4 := date_days(.{ d.year, 1, 4 });
    return d.year, (thursday - (jan4 + 4 - cast(s64) weekday(.{ d.year, 1, 4 }))) / 7 + 1;
}

// Validates and converts an ISO week date. Example: `d, error := date_from_iso_week(2024, 1, .MONDAY)`.
date_from_iso_week :: (year: s64, week: s64, day: Weekday) -> Date, Time_Error {
    if year < -9999 || year > 9999 || week < 1 || week > 53 || cast(s64) day < 1 || cast(s64) day > 7 return .{}, .{ .INVALID_VALUE, "invalid ISO week date", -1 };

    jan4 := Date.{ cast(s16) year, 1, 4 };
    result, e := checked_day(date_days(jan4) - cast(s64) weekday(jan4) + 1 + (week - 1) * 7 + cast(s64) day - 1);

    if e.kind != .NONE return .{}, e;

    y, w := iso_week(result);
    if y != year || w != week return .{}, .{ .INVALID_VALUE, "ISO week does not exist", -1 };

    return result, .{};
}

// Returns the first day of the validated date's month. Example: `d := first_of_month(value)`.
first_of_month :: (value: Date) -> Date {
    return .{ value.year, value.month, 1 };
}

// Returns the last day of the validated date's month. Example: `d := last_of_month(value)`.
last_of_month :: (value: Date) -> Date {
    return .{ value.year, value.month, cast(s8) month_length(value.year, value.month) };
}

// Finds occurrence 1..5 or -1..-5 from the end; missing occurrences fail. Example: `d, error := nth_weekday_of_month(2024, 3, .SUNDAY, 2)`.
nth_weekday_of_month :: (year: s64, month: s64, day: Weekday, nth: s64) -> Date, Time_Error {
    first, e := date(year, month, 1);
    if e.kind != .NONE return .{}, e;
    if nth == 0 || nth < -5 || nth > 5 || cast(s64) day < 1 || cast(s64) day > 7 return .{}, .{ .INVALID_VALUE, "invalid weekday occurrence", -1 };

    n := month_length(year, month);
    d: s64;

    if nth > 0 d = 1 + modulo(cast(s64) day - cast(s64) weekday(first), 7) + (nth - 1) * 7;
    else d = n - modulo(cast(s64) weekday(last_of_month(first)) - cast(s64) day, 7) + (nth + 1) * 7;

    return date(year, month, d);
}

// Applies years/months with month-end policy, then weeks/days; clock units fail. Example: `d, error := add(value, Span.{months = 1}, .REJECT)`.
add :: (value: Date, amount: Span, overflow := Overflow.CONSTRAIN) -> Date, Time_Error {
    if clock_nonzero(amount) return .{}, .{ .INVALID_VALUE, "date arithmetic rejects clock units", -1 };

    y: s64 = value.year;
    m: s64 = value.month;
    months := y * 12 + m - 1 + cast(s64) amount.years * 12 + amount.months;

    y = floor_div(months, 12);
    m = modulo(months, 12) + 1;
    if y < -9999 || y > 9999 return .{}, range_error();

    d: s64 = value.day;
    n := month_length(y, m);

    if d > n {
        if overflow == .REJECT return .{}, .{ .INVALID_VALUE, "day absent in target month", -1 };
        d = n;
    }

    shifted := Date.{ cast(s16) y, cast(s8) m, cast(s8) d };

    return checked_day(date_days(shifted) + cast(s64) amount.weeks * 7 + amount.days);
}

// Applies calendar units before exact clock units, carrying across midnight. Example: `dt, error := add(value, Span.{days = 1, hours = 2})`.
add :: (value: Date_Time, amount: Span, overflow := Overflow.CONSTRAIN) -> Date_Time, Time_Error {
    calendar := amount;

    clear_clock(*calendar);
    d, e := add(value.date, calendar, overflow);
    if e.kind != .NONE return .{}, e;

    total := clock_ns(amount) + Basic.to_s128(time_ns(value.time));
    q, r := wide_div(total, DAY_NS);

    if r < 0 {
        q -= Basic.to_s128(1);
        r += DAY_NS;
    }

    if q < Basic.to_s128(-8000000) || q > Basic.to_s128(8000000) return .{}, range_error();

    d, e = checked_day(date_days(d) + narrow(q));
    if e.kind != .NONE return .{}, e;

    return .{ d, clock_from_ns(r) }, .{};
}

// Adds clock units modulo one day; all calendar units fail. Example: `t, error := add(value, Span.{hours = 25})`.
add :: (value: Time_Of_Day, amount: Span) -> Time_Of_Day, Time_Error {
    if calendar_nonzero(amount) return .{}, .{ .INVALID_VALUE, "clock arithmetic rejects calendar units", -1 };
    total := clock_ns(amount) + Basic.to_s128(time_ns(value));
    _, rem := wide_div(total, DAY_NS);
    return clock_from_ns(modulo(rem, DAY_NS)), .{};
}

#scope_module

month_length :: (year: s64, month: s64) -> s64 {
    if month == 2 return ifx is_leap_year(year) then 29 else 28;
    if month == 4 || month == 6 || month == 9 || month == 11 return 30;
    return 31;
}

date_days :: (d: Date) -> s64 {
    y: s64 = d.year;
    m: s64 = d.month;

    if m <= 2 y -= 1;

    era := floor_div(y, 400);
    yo := y - era * 400;
    mp := m + (ifx m > 2 then -3 else 9);

    return era * 146097 + yo * 365 + yo / 4 - yo / 100 + (153 * mp + 2) / 5 + d.day - 1 - 719468;
}

civil_day :: (days: s64) -> Date {
    z := days + 719468;
    era := floor_div(z, 146097);
    doe := z - era * 146097;
    yo := (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    y := yo + era * 400;
    doy := doe - (yo * 365 + yo / 4 - yo / 100);
    mp := (5 * doy + 2) / 153;
    d := doy - (153 * mp + 2) / 5 + 1;
    m := mp + (ifx mp < 10 then 3 else -9);

    if m <= 2 y += 1;

    return .{ cast,trunc(s16) y, cast,trunc(s8) m, cast,trunc(s8) d };
}

checked_day :: (days: s64) -> Date, Time_Error {
    if days < date_days(.{ -9999, 1, 1 }) || days > date_days(.{ 9999, 12, 31 }) return .{}, range_error();
    return civil_day(days), .{};
}

time_ns :: (t: Time_Of_Day) -> s64 {
    return ((cast(s64) t.hour * 60 + t.minute) * 60 + t.second) * NS + t.nanosecond;
}

clock_from_ns :: (n: s64) -> Time_Of_Day {
    return .{ cast(s8) (n / (3600 * NS)), cast(s8) (n / (60 * NS) % 60), cast(s8) (n / NS % 60), cast(s32) (n % NS) };
}
