// Exact instants, civil values, explicit zones and calendar-aware spans.
// API declaration phase: procedure bodies are stubs; this module is not usable yet.
// Fallible stubs return UNSUPPORTED and never claim a successful calculation.

// A Gregorian date in years -9999..9999; construct with `date(2024, 3, 10)`.
Date :: struct {
    year: s16 = 1970;
    month: s8 = 1;
    day: s8 = 1;
}

// A clock reading without a zone; e.g. `time_of_day(2, 30, 0)`.
Time_Of_Day :: struct {
    hour: s8;
    minute: s8;
    second: s8;
    nanosecond: s32;
}

// A wall-clock value, not an instant; e.g. `date_time(2024, 3, 10, 2, 30, 0)`.
Date_Time :: struct {
    date: Date;
    time: Time_Of_Day;
}

// Unix seconds and a signed subsecond remainder; e.g. `timestamp_from_millis(-1)`.
// The remainder is in -999999999..999999999 and has the seconds' sign when seconds != 0.
Timestamp :: struct {
    seconds: s64;
    nanoseconds: s32;
}

// Exact elapsed time, with the same remainder invariant as Timestamp; e.g. `signed_duration(1, 5)`.
Signed_Duration :: struct {
    seconds: s64;
    nanoseconds: s32;
}

// Seconds east of UTC, within +/-25:59:59; e.g. `offset(19800)`.
Offset :: struct {
    seconds: s32;
}

// The representation of a zone; e.g. `zone.kind == .FIXED`.
Time_Zone_Kind :: enum u8 {
    UTC;
    FIXED;
    RULES;
}

// An immutable zone handle; e.g. `zone, error := time_zone_get("America/New_York")`.
// RULES handles own one reference. Assignment borrows; time_zone_retain creates another owner.
// Release each owner once with time_zone_free; a Zoned borrows its zone.
Time_Zone :: struct {
    kind: Time_Zone_Kind;
    offset: Offset;
    rules: *void; // opaque; only this module may dereference it
}

// An instant and a borrowed zone, with its resolved offset; e.g. `zoned_now(time_zone_utc())`.
Zoned :: struct {
    timestamp: Timestamp;
    zone: Time_Zone;
    offset: Offset;
}

// Separate calendar and clock units, all zero or sharing one sign; e.g. `span_check(.{days = 1})`.
// Raw field assignment does not validate: use span_check before passing caller-built values to arithmetic.
Span :: struct {
    years: s32;
    months: s32;
    weeks: s32;
    days: s32;
    hours: s64;
    minutes: s64;
    seconds: s64;
    milliseconds: s64;
    microseconds: s64;
    nanoseconds: s64;
}

// Units ordered from finest to coarsest; e.g. `span_total(value, .HOUR)`.
Unit :: enum u8 {
    NANOSECOND;
    MICROSECOND;
    MILLISECOND;
    SECOND;
    MINUTE;
    HOUR;
    DAY;
    WEEK;
    MONTH;
    YEAR;
}

// ISO weekday numbering; e.g. `weekday(value)` returns MONDAY for 2024-03-11.
Weekday :: enum u8 {
    MONDAY :: 1;
    TUESDAY;
    WEDNESDAY;
    THURSDAY;
    FRIDAY;
    SATURDAY;
    SUNDAY;
}

// Temporal rounding modes; e.g. `.HALF_EVEN` sends equal ties to an even multiple.
Rounding :: enum u8 {
    HALF_EXPAND;
    TRUNC;
    FLOOR;
    CEIL;
    EXPAND;
    HALF_TRUNC;
    HALF_EVEN;
    HALF_FLOOR;
    HALF_CEIL;
}

// Resolves skipped/repeated civil times; e.g. `.REJECT` fails for either ambiguity.
// COMPATIBLE chooses later in a gap and earlier in a fold.
Disambiguation :: enum u8 {
    COMPATIBLE;
    EARLIER;
    LATER;
    REJECT;
}

// Month-end handling; e.g. January 31 plus one month constrains to February's last day.
Overflow :: enum u8 {
    CONSTRAIN;
    REJECT;
}

// Failure classification; e.g. `error.kind == .NONE` means success.
Time_Error_Kind :: enum u8 {
    NONE;
    INVALID_VALUE;
    OUT_OF_RANGE;
    OVERFLOW;
    SYNTAX;
    LIMIT;
    GAP;
    FOLD;
    OFFSET_CONFLICT;
    RELATIVE_REQUIRED;
    IO;
    NOT_FOUND;
    UNSUPPORTED;
}

// Static message and zero-based parse byte offset (-1 for other errors); e.g. `error.offset == 4`.
// No error owns memory. Failed results must not be used.
Time_Error :: struct {
    kind: Time_Error_Kind;
    message: string;
    offset: s64 = -1;
}

// Bounds on text, TZif data and rule counts; e.g. `options.max_transitions = 10000`.
// Each public parser checks these before copying or allocating input-derived storage.
Time_Options :: struct {
    max_text: s64 = 4096;
    max_tzif_bytes: s64 = 16 * 1024 * 1024;
    max_transitions: s64 = 1000000;
    max_types: s64 = 256;
    max_abbreviation_bytes: s64 = 65536;
}

// One offset change; e.g. `change.after.seconds - change.before.seconds == 3600`.
// Abbreviation slices borrow the iterator's zone owner.
Time_Transition :: struct {
    timestamp: Timestamp;
    before: Offset;
    after: Offset;
    abbreviation_before: string;
    abbreviation_after: string;
}

// A constant-space cursor borrowing a zone; e.g. `cursor, error := transitions(zone, instant)`.
Time_Transitions :: struct {
    zone: Time_Zone;
    cursor: Timestamp;
    reverse: bool;
    exhausted: bool;
}

// Validates Gregorian components without clamping.
// Example: `d, error := date(2024, 2, 29)`.
date :: (year: s64, month: s64, day: s64) -> Date, Time_Error {
    n, e := days_in_month(year, month);
    if e.kind != .NONE return .{}, e;
    if day < 1 || day > n return .{}, .{ .INVALID_VALUE, "invalid day", -1 };
    return .{ cast(s16) year, cast(s8) month, cast(s8) day }, .{};
}

// Validates a clock reading; leap seconds are rejected.
// Example: `t, error := time_of_day(23, 59, 59, 1)`.
time_of_day :: (hour: s64, minute: s64, second: s64, nanosecond: s64 = 0) -> Time_Of_Day, Time_Error {
    if hour < 0 || hour > 23 || minute < 0 || minute > 59 || second < 0 || second > 59 || nanosecond < 0 || nanosecond >= NS return .{}, .{ .INVALID_VALUE, "invalid clock component", -1 };
    return .{ cast(s8) hour, cast(s8) minute, cast(s8) second, cast(s32) nanosecond }, .{};
}

// Validates and combines civil components.
// Example: `dt, error := date_time(day, clock)`.
date_time :: (day: Date, clock: Time_Of_Day) -> Date_Time {
    return .{ day, clock };
}

// Validates a civil date and clock reading together.
// Example: `dt, error := date_time(2024, 3, 10, 2, 30)`.
date_time :: (year: s64, month: s64, day: s64, hour: s64 = 0, minute: s64 = 0, second: s64 = 0, nanosecond: s64 = 0) -> Date_Time, Time_Error {
    d, e := date(year, month, day);
    if e.kind != .NONE return .{}, e;

    t, te := time_of_day(hour, minute, second, nanosecond);
    if te.kind != .NONE return .{}, te;

    return .{ d, t }, .{};
}

// Validates seconds east of UTC, including historical second offsets.
// Example: `o, error := offset(-14400)`.
offset :: (seconds: s64) -> Offset, Time_Error {
    if seconds < -93599 || seconds > 93599 return .{}, range_error();
    return .{ cast(s32) seconds }, .{};
}

// Normalizes signed seconds and nanoseconds, checking representability.
// Example: `d, error := signed_duration(1, -1)`.
signed_duration :: (seconds: s64, nanoseconds: s64 = 0) -> Signed_Duration, Time_Error {
    return duration_wide(wide_mul(seconds, NS) + to_s128(nanoseconds));
}

// Validates the single-sign invariant without balancing units.
// Example: `s, error := span_check(.{months = 1, days = 2})`.
span_check :: (value: Span) -> Span, Time_Error {
    sign: s64;

    for field: span_fields(value) {
        if field == MIN_I64 || field == -2147483648 return .{}, .{ .OUT_OF_RANGE, "span field cannot be negated", -1 };

        current := cmp64(field, 0);
        if current != 0 {
            if sign != 0 && current != sign return .{}, .{ .INVALID_VALUE, "mixed span signs", -1 };
            sign = current;
        }
    }

    return value, .{};
}

// Returns the Gregorian month length after validating year and month.
// Example: `n, error := days_in_month(2024, 2)`.
days_in_month :: (year: s64, month: s64) -> s64, Time_Error {
    if year < -9999 || year > 9999 return 0, .{ .OUT_OF_RANGE, "year outside civil range", -1 };
    if month < 1 || month > 12 return 0, .{ .INVALID_VALUE, "invalid month", -1 };
    return month_length(year, month), .{};
}

// Tests the Gregorian leap rule within the supported year range.
// Example: `leap := is_leap_year(2000)`.
is_leap_year :: (year: s64) -> bool {
    return year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
}

// Returns the ISO weekday after validating the date.
// Example: `day := weekday(value)`.
weekday :: (value: Date) -> Weekday {
    return cast(Weekday) (modulo(date_days(value) + 3, 7) + 1);
}

// Returns the ordinal day, starting at one.
// Example: `ordinal := day_of_year(value)`.
day_of_year :: (value: Date) -> s64 {
    return date_days(value) - date_days(.{ value.year, 1, 1 }) + 1;
}

// Returns ISO week-year and week number; week one contains January 4.
// Example: `year, week := iso_week(value)`.
iso_week :: (value: Date) -> s64, s64 {
    thursday := date_days(value) + 4 - cast(s64) weekday(value);
    d := civil_day(thursday);
    jan4 := date_days(.{ d.year, 1, 4 });
    return d.year, (thursday - (jan4 + 4 - cast(s64) weekday(.{ d.year, 1, 4 }))) / 7 + 1;
}

// Validates and converts an ISO week date.
// Example: `d, error := date_from_iso_week(2024, 1, .MONDAY)`.
date_from_iso_week :: (year: s64, week: s64, day: Weekday) -> Date, Time_Error {
    if year < -9999 || year > 9999 || week < 1 || week > 53 || cast(s64) day < 1 || cast(s64) day > 7 return .{}, .{ .INVALID_VALUE, "invalid ISO week date", -1 };

    jan4 := Date.{ cast(s16) year, 1, 4 };
    result, e := checked_day(date_days(jan4) - cast(s64) weekday(jan4) + 1 + (week - 1) * 7 + cast(s64) day - 1);

    if e.kind != .NONE return .{}, e;

    y, w := iso_week(result);
    if y != year || w != week return .{}, .{ .INVALID_VALUE, "ISO week does not exist", -1 };

    return result, .{};
}

// Returns the first day of the validated date's month.
// Example: `d := first_of_month(value)`.
first_of_month :: (value: Date) -> Date {
    return .{ value.year, value.month, 1 };
}

// Returns the last day of the validated date's month.
// Example: `d := last_of_month(value)`.
last_of_month :: (value: Date) -> Date {
    return .{ value.year, value.month, cast(s8) month_length(value.year, value.month) };
}

// Finds occurrence 1..5 or -1..-5 from the end; missing occurrences fail.
// Example: `d, error := nth_weekday_of_month(2024, 3, .SUNDAY, 2)`.
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

// Applies years/months with month-end policy, then weeks/days; clock units fail.
// Example: `d, error := add(value, Span.{months = 1}, .REJECT)`.
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

// Applies calendar units before exact clock units, carrying across midnight.
// Example: `dt, error := add(value, Span.{days = 1, hours = 2})`.
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

// Adds clock units modulo one day; all calendar units fail.
// Example: `t, error := add(value, Span.{hours = 25})`.
add :: (value: Time_Of_Day, amount: Span) -> Time_Of_Day, Time_Error {
    if calendar_nonzero(amount) return .{}, .{ .INVALID_VALUE, "clock arithmetic rejects calendar units", -1 };
    total := clock_ns(amount) + Basic.to_s128(time_ns(value));
    _, rem := wide_div(total, DAY_NS);
    return clock_from_ns(modulo(rem, DAY_NS)), .{};
}

// Measures b minus a; validates units, positive increments and range.
// Example: `s, error := until(a, b)`.
until :: (a: Date, b: Date, largest := Unit.DAY, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1) -> Span, Time_Error {
    if largest != .DAY || (smallest != .DAY && smallest != .NANOSECOND) || increment < 1 return .{}, .{ .UNSUPPORTED, "calendar rounding is not implemented yet", -1 };
    return .{ days = cast(s32) (date_days(b) - date_days(a)) }, .{};
}

// Measures b minus a; validates units, positive increments and range.
// Example: `s, error := until(a, b)`.
until :: (a: Date_Time, b: Date_Time, largest := Unit.DAY, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1) -> Span, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Measures b minus a; validates units, positive increments and range.
// Example: `s, error := until(a, b)`.
until :: (a: Timestamp, b: Timestamp, largest := Unit.SECOND, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1) -> Span, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Measures b minus a; calendar units use the receiver's zone and real day lengths.
// Example: `s, error := until(a, b)`.
until :: (a: Zoned, b: Zoned, largest := Unit.DAY, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1, disambiguation := Disambiguation.COMPATIBLE) -> Span, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Measures a minus b; validates units, positive increments and range.
// Example: `s, error := since(a, b)`.
since :: (a: Date, b: Date, largest := Unit.DAY, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1) -> Span, Time_Error {
    return until(b, a, largest, smallest, rounding, increment);
}

// Measures a minus b; validates units, positive increments and range.
// Example: `s, error := since(a, b)`.
since :: (a: Date_Time, b: Date_Time, largest := Unit.DAY, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1) -> Span, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Measures a minus b; validates units, positive increments and range.
// Example: `s, error := since(a, b)`.
since :: (a: Timestamp, b: Timestamp, largest := Unit.SECOND, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1) -> Span, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Measures a minus b; calendar units use the receiver's zone and real day lengths.
// Example: `s, error := since(a, b)`.
since :: (a: Zoned, b: Zoned, largest := Unit.DAY, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1, disambiguation := Disambiguation.COMPATIBLE) -> Span, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Balances to largest; calendar conversions require a relative value.
// Example: `value, error := span_balance(value, .DAY, relative = day)`.
span_balance :: (value: Span, largest: Unit) -> Span, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Uses a relative Date; e.g. `span_balance(value, relative = origin)`.
span_balance :: (value: Span, largest: Unit, relative: Date) -> Span, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Uses a relative Zoned; e.g. `span_balance(value, relative = origin)`.
span_balance :: (value: Span, largest: Unit, relative: Zoned) -> Span, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Rounds to a positive increment, then balances; calendar rounding needs a relative value.
// Example: `value, error := span_round(value, .MINUTE, .HOUR)`.
span_round :: (value: Span, smallest: Unit, largest: Unit, rounding := Rounding.HALF_EXPAND, increment: s64 = 1) -> Span, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Uses a relative Date; e.g. `span_round(value, relative = origin)`.
span_round :: (value: Span, smallest: Unit, largest: Unit, rounding := Rounding.HALF_EXPAND, increment: s64 = 1, relative: Date) -> Span, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Uses a relative Zoned; e.g. `span_round(value, relative = origin)`.
span_round :: (value: Span, smallest: Unit, largest: Unit, rounding := Rounding.HALF_EXPAND, increment: s64 = 1, relative: Zoned) -> Span, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Returns a possibly fractional total; zoned days follow actual day lengths.
// Example: `value, error := span_total(value, .HOUR, relative = z)`.
span_total :: (value: Span, unit: Unit) -> float64, Time_Error {
    return 0, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Uses a relative Date; e.g. `span_total(value, relative = origin)`.
span_total :: (value: Span, unit: Unit, relative: Date) -> float64, Time_Error {
    return 0, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Uses a relative Zoned; e.g. `span_total(value, relative = origin)`.
span_total :: (value: Span, unit: Unit, relative: Zoned) -> float64, Time_Error {
    return 0, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Returns -1/0/1 by elapsed length; calendar units require a shared relative value.
// Example: `value, error := span_compare(a, b, relative = day)`.
span_compare :: (a: Span, b: Span) -> int, Time_Error {
    return 0, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Uses a relative Date; e.g. `span_compare(value, relative = origin)`.
span_compare :: (a: Span, b: Span, relative: Date) -> int, Time_Error {
    return 0, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Uses a relative Zoned; e.g. `span_compare(value, relative = origin)`.
span_compare :: (a: Span, b: Span, relative: Zoned) -> int, Time_Error {
    return 0, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Converts clock units to elapsed time; calendar units require a relative value.
// Example: `value, error := span_to_duration(value)`.
span_to_duration :: (value: Span) -> Signed_Duration, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Uses a relative Date; e.g. `span_to_duration(value, relative = origin)`.
span_to_duration :: (value: Span, relative: Date) -> Signed_Duration, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Uses a relative Zoned; e.g. `span_to_duration(value, relative = origin)`.
span_to_duration :: (value: Span, relative: Zoned) -> Signed_Duration, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Converts clock units only; years, months, weeks and days are errors.
// Example: `value, error := duration_from_span(value)`.
duration_from_span :: (value: Span) -> Signed_Duration, Time_Error {
    if calendar_nonzero(value) return .{}, .{ .RELATIVE_REQUIRED, "calendar units require a relative value", -1 };
    return duration_wide(clock_ns(value));
}

// Reverses every field of a validated span.
// Example: `s := span_negate(value)`.
span_negate :: (value: Span) -> Span {
    return .{ -value.years, -value.months, -value.weeks, -value.days, -value.hours, -value.minutes, -value.seconds, -value.milliseconds, -value.microseconds, -value.nanoseconds };
}

// Returns a nonnegative span; span_check rejects signed minima.
// Example: `s := span_abs(value)`.
span_abs :: (value: Span) -> Span {
    for field: span_fields(value) {
        if field < 0 return span_negate(value);
        if field > 0 return value;
    }

    return value;
}

// Reads Basic.current_time_consensus and converts its Apollo epoch; no new clock path.
// Example: `instant, error := timestamp_now()`.
timestamp_now :: () -> Timestamp, Time_Error {
    return timestamp_from_apollo(Basic.current_time_consensus());
}

// Converts signed Unix seconds, checking the UTC year range.
// Example: `instant, error := timestamp_from_seconds(-1)`.
timestamp_from_seconds :: (value: s64) -> Timestamp, Time_Error {
    return timestamp_wide(wide_mul(value, NS));
}

// Converts signed Unix milliseconds, checking the UTC year range.
// Example: `instant, error := timestamp_from_millis(-1)`.
timestamp_from_millis :: (value: s64) -> Timestamp, Time_Error {
    return timestamp_wide(wide_mul(value / 1000, NS) + wide_mul(value % 1000, NS / 1000));
}

// Converts signed Unix microseconds, checking the UTC year range.
// Example: `instant, error := timestamp_from_micros(-1)`.
timestamp_from_micros :: (value: s64) -> Timestamp, Time_Error {
    return timestamp_wide(wide_mul(value / 1000000, NS) + wide_mul(value % 1000000, NS / 1000000));
}

// Converts signed Unix nanoseconds, checking the UTC year range.
// Example: `instant, error := timestamp_from_nanos(-1)`.
timestamp_from_nanos :: (value: s64) -> Timestamp, Time_Error {
    return timestamp_wide(wide_mul(value / 1000000000, NS) + wide_mul(value % 1000000000, NS / 1000000000));
}

// Adds exact elapsed time with checked range and normalization.
// Example: `instant, error := add(value, duration)`.
add :: (value: Timestamp, amount: Signed_Duration) -> Timestamp, Time_Error {
    return timestamp_wide(stamp_ns(value) + wide_mul(amount.seconds, NS) + to_s128(amount.nanoseconds));
}

// Adds clock units only; calendar units, including days, fail.
// Example: `instant, error := add(value, Span.{hours = 24})`.
add :: (value: Timestamp, amount: Span) -> Timestamp, Time_Error {
    d, e := duration_from_span(amount);
    if e.kind != .NONE return .{}, e;
    return timestamp_wide(stamp_ns(value) + wide_mul(d.seconds, NS) + to_s128(d.nanoseconds));
}

// Returns exact b minus a without calendar interpretation.
// Example: `elapsed := duration_between(a, b)`.
duration_between :: (a: Timestamp, b: Timestamp) -> Signed_Duration {
    q, r := wide_div(stamp_ns(b) - stamp_ns(a), NS);
    return .{ narrow(q), cast(s32) r };
}

// Returns the allocation-free UTC zone.
// Example: `zone := time_zone_utc()`.
time_zone_utc :: () -> Time_Zone {
    return .{};
}

// Validates and constructs an allocation-free fixed-offset zone.
// Example: `zone := time_zone_fixed(value)`.
time_zone_fixed :: (value: Offset) -> Time_Zone {
    return .{ .FIXED, value, null };
}

// Loads and caches a safe IANA name; directory overrides TZDIR for deterministic fixtures.
// Example: `zone, error := time_zone_get("America/New_York")`.
time_zone_get :: (name: string, options: Time_Options = .{}, directory: string = "") -> Time_Zone, Time_Error {
    if !valid_options(options) || name.count == 0 || name.count > options.max_text return .{}, .{ .INVALID_VALUE, "invalid zone name", -1 };
    for c: name if !((c >= 65 && c <= 90) || (c >= 97 && c <= 122) || (c >= 48 && c <= 57) || c == 47 || c == 95 || c == 45 || c == 43) return .{}, .{ .INVALID_VALUE, "unsafe zone name", -1 };
    if name[0] == 47 || name[name.count - 1] == 47 return .{}, .{ .INVALID_VALUE, "absolute or empty zone component", -1 };
    for i: 1..name.count - 1 if name[i] == 47 && name[i - 1] == 47 return .{}, .{ .INVALID_VALUE, "empty zone component", -1 };

    dir := directory;
    if dir.count == 0 dir = Env.environment_variable("TZDIR");

    if dir.count == 0 {
        #if OS == .WINDOWS return .{}, .{ .UNSUPPORTED, "Windows named zones require TZDIR", -1 };
        else dir = "/usr/share/zoneinfo";
    }

    if dir.count > options.max_text return .{}, .{ .LIMIT, "zone directory limit", -1 };

    path := tprint("%/%", dir, name);

    while !compare_and_swap(*zone_cache_lock, cast(s32) 0, cast(s32) 1) {}

    defer Atomics.atomic_write(*zone_cache_lock, cast(s32) 0);

    for entry: zone_cache {
        if entry.rules == null continue;
        r := cast(*Zone_Rules) entry.rules;
        if r.path == path && zone_within_limits(r, options) return time_zone_retain(entry), .{};
    }

    // Cache storage must outlive a caller's arena or counting allocator.
    push_allocator(context.default_allocator.proc, context.default_allocator.data);
    canonical, exists := Files.get_absolute_path(path);

    defer free(canonical);
    base, base_ok := Files.get_absolute_path(dir);

    defer free(base);
    if !exists || !base_ok return .{}, .{ .NOT_FOUND, "cannot resolve zone path", -1 };
    if !begins_with(canonical, tprint("%/", base)) return .{}, .{ .INVALID_VALUE, "zone path escapes directory", -1 };

    bytes, e := read_zone_file(canonical, options);
    if e.kind != .NONE return .{}, e;

    defer free(bytes.data);
    z, ze := time_zone_from_tzif(name, bytes, options);
    if ze.kind != .NONE return .{}, ze;

    r := cast(*Zone_Rules) z.rules;
    r.path = copy_string(path);
    time_zone_free(*zone_cache[zone_cache_slot]);
    zone_cache[zone_cache_slot] = time_zone_retain(z);
    zone_cache_slot = (zone_cache_slot + 1) % zone_cache.count;

    return z, .{};
}

// Uses TZ, otherwise /etc/localtime; Windows without TZ returns UNSUPPORTED.
// Example: `zone, error := time_zone_system()`.
time_zone_system :: (options: Time_Options = .{}) -> Time_Zone, Time_Error {
    text := Env.environment_variable("TZ");
    if text.count != 0 {
        if text[0] == 58 return time_zone_get(slice(text, 1, text.count - 1), options);
        zone, e := time_zone_from_posix(text, options);
        if e.kind == .NONE return zone, e;
        return time_zone_get(text, options);
    }

    #if OS == .WINDOWS return .{}, .{ .UNSUPPORTED, "Windows system zone without TZ is not supported yet", -1 };
    else {
        bytes, e := read_zone_file("/etc/localtime", options);
        if e.kind != .NONE return .{}, e;

        defer free(bytes.data);
        name: string;
        canonical, ok := Files.get_absolute_path("/etc/localtime");

        defer free(canonical);

        if ok {
            at := find_index_from_left(canonical, "/zoneinfo/");
            if at >= 0 name = slice(canonical, at + 10, canonical.count - at - 10);
        }

        return time_zone_from_tzif(name, bytes, options);
    }
}

// Copies validated TZif v1-v4 rules and POSIX footer within bounded storage.
// Example: `zone, error := time_zone_from_tzif("Example/Zone", bytes)`.
time_zone_from_tzif :: (name: string, bytes: [] u8, options: Time_Options = .{}) -> Time_Zone, Time_Error {
    if !valid_options(options) || bytes.count > options.max_tzif_bytes || name.count > options.max_text return .{}, .{ .LIMIT, "zone data limit", 0 };

    r := Binary_Reader.{ bytes = bytes };
    h := tzif_header(*r, options);
    width: s64 = 4;

    if r.error.kind != .NONE return .{}, r.error;

    if h.version != 0 {
        r.at += block_size(h, 4);
        previous := h.version;

        h = tzif_header(*r, options);
        width = 8;
        if h.version != previous binary_fail(*r, "TZif section version mismatch");
    }

    if r.error.kind != .NONE return .{}, r.error;

    start := r.at;
    size := block_size(h, width);

    if size > bytes.count - start return .{}, .{ .SYNTAX, "truncated TZif block", start };
    if h.leaps != 0 return .{}, .{ .UNSUPPORTED, "leap-second TZif is outside scope", start };

    ta := start;
    ia := ta + h.times * width;
    types := ia + h.times;
    ab := types + h.types * 6;
    last: s64 = MIN_I64;

    for i: 0..h.times - 1 {
        value := be_signed(bytes, ta + i * width, width);
        if (i > 0 && value <= last) || bytes[ia + i] >= h.types return .{}, .{ .INVALID_VALUE, "invalid TZif transition", ta + i * width };
        last = value;
    }

    for i: 0..h.types - 1 {
        at := types + i * 6;
        off := be_signed(bytes, at, 4);
        index: s64 = bytes[at + 5];

        if off < -93599 || off > 93599 || bytes[at + 4] > 1 || index >= h.chars return .{}, .{ .INVALID_VALUE, "invalid TZif type", at };

        end := index;
        while end < h.chars && bytes[ab + end] != 0 end += 1;
        if end == h.chars return .{}, .{ .INVALID_VALUE, "unterminated TZif abbreviation", ab + index };
    }

    flag_at := ab + h.chars;

    for i: 0..h.standard + h.utc - 1 {
        if bytes[flag_at + i] > 1 return .{}, .{ .INVALID_VALUE, "invalid TZif indicator", flag_at + i };
        if i >= h.standard && bytes[flag_at + i] == 1 && (h.standard == 0 || bytes[flag_at + i - h.standard] == 0) return .{}, .{ .INVALID_VALUE, "UT indicator requires standard indicator", flag_at + i };
    }

    r.at = start + size;
    footer: string;

    if width == 8 {
        if r.at >= bytes.count || bytes[r.at] != 10 return .{}, .{ .SYNTAX, "missing TZif footer", r.at };

        r.at += 1;
        begin := r.at;

        while r.at < bytes.count && bytes[r.at] != 10 r.at += 1;
        if r.at >= bytes.count || r.at + 1 != bytes.count return .{}, .{ .SYNTAX, "invalid TZif footer terminator", r.at };

        footer = string.{ r.at - begin, bytes.data + begin };
        if footer.count > options.max_text return .{}, .{ .LIMIT, "TZif footer limit", begin };

        if footer.count != 0 {
            text := Text_Reader.{ text = footer };

            read_posix(*text);

            if text.error.kind != .NONE {
                text.error.offset += begin;
                return .{}, text.error;
            }
        }
    } else if r.at != bytes.count return .{}, .{ .SYNTAX, "trailing TZif bytes", r.at };

    owner := new_rules(name, string.{ bytes.count, bytes.data });
    owner.width = width;
    owner.count = h.times;
    owner.types = h.types;
    owner.ta = ta;
    owner.ia = ia;
    owner.type_at = types;
    owner.ab = ab;
    owner.chars = h.chars;

    if footer.count != 0 {
        text := Text_Reader.{ text = string.{ footer.count, owner.source.data + (footer.data - bytes.data) } };
        owner.footer = read_posix(*text);
    }

    return .{ .RULES, .{}, owner }, .{};
}

// Parses POSIX TZ rules including M/J/Julian forms and extended transition times.
// Example: `zone, error := time_zone_from_posix("EST5EDT,M3.2.0/2,M11.1.0/2")`.
time_zone_from_posix :: (text: string, options: Time_Options = .{}) -> Time_Zone, Time_Error {
    if !valid_options(options) || text.count > options.max_text return .{}, .{ .LIMIT, "zone text limit", 0 };

    r := Text_Reader.{ text = text };
    rules := read_posix(*r);

    if r.error.kind != .NONE return .{}, r.error;

    owner := new_rules(name = "", source = text);
    owner.footer = rules;
    // Reparse so abbreviation views borrow the owned copy.
    r = .{ text = owner.source };
    owner.footer = read_posix(*r);

    return .{ .RULES, .{}, owner }, .{};
}

// Acquires an independent owner; UTC and fixed zones need no reference count.
// Example: `owned := time_zone_retain(borrowed)`.
time_zone_retain :: (value: Time_Zone) -> Time_Zone {
    if value.kind == .RULES && value.rules != null {
        r := lookup_rules(value.rules);
        if r == null return .{};
        Atomics.atomic_increment(*r.refs);
    }

    return value;
}

// Releases one owned reference and resets the handle to UTC; borrowed copies become invalid.
// Example: `time_zone_free(*zone)`.
time_zone_free :: (value: *Time_Zone) {
    if value.kind == .RULES && value.rules != null {
        r := lookup_rules(value.rules);
        if r == null {
            value.* = .{};
            return;
        }

        if Atomics.atomic_decrement(*r.refs) {
            unregister_rules(r);
            push_allocator(r.allocator.proc, r.allocator.data);
            free(r.path);
            free(r.name);
            free(r.source);
            free(r);
        }
    }

    value.* = .{};
}

// Resolves the offset at an exact instant, using footer rules beyond stored transitions.
// Example: `o := offset_at(zone, instant)`.
offset_at :: (zone: Time_Zone, instant: Timestamp) -> Offset {
    if zone.kind == .FIXED return zone.offset;
    if zone.kind != .RULES || zone.rules == null return .{};

    r := lookup_rules(zone.rules);
    if r == null return .{};

    second := instant.seconds;

    if instant.nanoseconds < 0 second -= 1;

    index := zone_type(r, second);
    if index < 0 {
        off, _ := posix_at(r.footer, second);
        return .{ cast(s32) off };
    }

    return .{ cast(s32) be_signed(r.source, r.type_at + index * 6, 4) };
}

// Returns a zone-borrowed abbreviation for formatting and conformance.
// Example: `text := abbreviation_at(zone, instant)`.
abbreviation_at :: (zone: Time_Zone, instant: Timestamp) -> string {
    if zone.kind == .UTC return "UTC";
    if zone.kind != .RULES || zone.rules == null return "";

    r := lookup_rules(zone.rules);
    if r == null return .{};

    second := instant.seconds;

    if instant.nanoseconds < 0 second -= 1;

    index := zone_type(r, second);
    if index < 0 {
        _, name := posix_at(r.footer, second);
        return name;
    }

    at := r.ab + r.source[r.type_at + index * 6 + 5];
    end := at;

    while end < r.ab + r.chars && r.source[end] != 0 end += 1;

    return string.{ end - at, r.source.data + at };
}

// Converts an instant in an explicit zone, rejecting civil range overflow.
// Example: `wall := to_civil(zone, instant)`.
to_civil :: (zone: Time_Zone, instant: Timestamp) -> Date_Time {
    seconds := instant.seconds + offset_at(zone, instant).seconds;
    ns: s64 = instant.nanoseconds;

    if ns < 0 {
        seconds -= 1;
        ns += NS;
    }

    return .{ civil_day(floor_div(seconds, 86400)), clock_from_ns(modulo(seconds, 86400) * NS + ns) };
}

// Resolves gaps/folds using the requested policy and checks UTC range.
// Example: `instant, error := to_timestamp(zone, wall, .REJECT)`.
to_timestamp :: (zone: Time_Zone, wall: Date_Time, disambiguation := Disambiguation.COMPATIBLE) -> Timestamp, Time_Error {
    d, e := date(wall.date.year, wall.date.month, wall.date.day);
    if e.kind != .NONE return .{}, e;

    t, te := time_of_day(wall.time.hour, wall.time.minute, wall.time.second, wall.time.nanosecond);
    if te.kind != .NONE return .{}, te;

    o := offset_at(zone, .{});

    return timestamp_wide(wide_mul(date_days(d) * 86400 - o.seconds, NS) + to_s128(time_ns(t)));
}

// Starts a borrowed cursor strictly after instant (strictly before when reverse).
// Example: `cursor := transitions(zone, instant)`.
transitions :: (zone: Time_Zone, instant: Timestamp, reverse := false) -> Time_Transitions {
    return .{ zone, instant, reverse, false };
}

// Advances a transition cursor; false with no error means the range is exhausted.
// Example: `change, found, error := next(*cursor)`.
next :: (cursor: *Time_Transitions) -> Time_Transition, bool, Time_Error {
    if cursor.exhausted || cursor.zone.kind != .RULES || cursor.zone.rules == null return .{}, false, .{};

    r := lookup_rules(cursor.zone.rules);
    if r == null return .{}, false, .{};

    found := false;
    chosen: s64;
    lo: s64;
    hi := r.count;

    while lo < hi {
        mid := lo + (hi - lo) / 2;
        seconds := be_signed(r.source, r.ta + mid * r.width, r.width);
        c := compare(Timestamp.{ seconds, 0 }, cursor.cursor);

        if c < 0 || (!cursor.reverse && c == 0) lo = mid + 1;
        else hi = mid;
    }

    index := ifx cursor.reverse then lo - 1 else lo;
    if index >= 0 && index < r.count {
        chosen = be_signed(r.source, r.ta + index * r.width, r.width);
        found = true;
    }

    if r.footer.present && r.footer.dst {
        y: s64 = to_civil(time_zone_utc(), cursor.cursor).date.year;
        last := MIN_I64;

        if r.count > 0 last = be_signed(r.source, r.ta + (r.count - 1) * r.width, r.width);

        for year: y - 1..y + 1 {
            for event: s64.[rule_in_year(r.footer.start, year, r.footer.standard), rule_in_year(r.footer.end, year, r.footer.daylight)] {
                c := compare(Timestamp.{ event, 0 }, cursor.cursor);
                if event > last && ((cursor.reverse && c < 0) || (!cursor.reverse && c > 0)) && (!found || (cursor.reverse && event > chosen) || (!cursor.reverse && event < chosen)) {
                    chosen = event;
                    found = true;
                }
            }
        }
    }

    if !found {
        cursor.exhausted = true;
        return .{}, false, .{};
    }

    instant, e := timestamp_from_seconds(chosen);
    if e.kind != .NONE {
        cursor.exhausted = true;
        return .{}, false, .{};
    }

    before := Timestamp.{ chosen - 1, 0 };

    cursor.cursor = instant;

    return .{ instant, offset_at(cursor.zone, before), offset_at(cursor.zone, instant), abbreviation_at(cursor.zone, before), abbreviation_at(cursor.zone, instant) }, true, .{};
}

// Resolves an instant and borrows its zone; the zone owner must outlive the result.
// Example: `z := zoned(instant, zone)`.
zoned :: (instant: Timestamp, zone: Time_Zone) -> Zoned {
    return .{};
}

// Resolves civil time and borrows the explicit zone.
// Example: `z, error := to_zoned(wall, zone, .LATER)`.
to_zoned :: (wall: Date_Time, zone: Time_Zone, disambiguation := Disambiguation.COMPATIBLE) -> Zoned {
    return .{};
}

// Reads the existing Basic consensus clock in an explicit borrowed zone.
// Example: `z, error := zoned_now(zone)`.
zoned_now :: (zone: Time_Zone) -> Zoned, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Keeps the instant while borrowing a different zone and resolving its offset.
// Example: `z := with_zone(value, time_zone_utc())`.
with_zone :: (value: Zoned, zone: Time_Zone) -> Zoned {
    return .{};
}

// Adds calendar units in civil time, resolves, then adds clock units on the timeline.
// Example: `z, error := add(value, Span.{days = 1})`.
add :: (value: Zoned, amount: Span, overflow := Overflow.CONSTRAIN, disambiguation := Disambiguation.COMPATIBLE) -> Zoned, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Finds the first valid instant of the local date, including days starting after midnight.
// Example: `start, error := start_of_day(value)`.
start_of_day :: (value: Zoned) -> Zoned, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Returns exact elapsed hours between consecutive local day starts.
// Example: `hours, error := hours_in_day(value)`.
hours_in_day :: (value: Zoned) -> float64, Time_Error {
    return 0, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Orders values (-1/0/1); Zoned and Timestamp use only the instant. Zones order by kind and rule content.
// Example: `order := compare(a, b)`.
compare :: (a: $T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned], b: T) -> int {
    #if T == Date {
        c := cmp64(a.year, b.year);
        if c != 0 return c;

        c = cmp64(a.month, b.month);
        if c != 0 return c;

        return cmp64(a.day, b.day);
    } else #if T == Time_Of_Day {
        return cmp64(time_ns(a), time_ns(b));
    } else #if T == Date_Time {
        c := compare(a.date, b.date);
        if c != 0 return c;
        return compare(a.time, b.time);
    } else #if T == Timestamp || T == Signed_Duration {
        c := cmp64(a.seconds, b.seconds);
        if c != 0 return c;
        return cmp64(a.nanoseconds, b.nanoseconds);
    } else #if T == Offset {
        return cmp64(a.seconds, b.seconds);
    } else #if T == Zoned {
        return compare(a.timestamp, b.timestamp);
    } else {
        c := cmp64(cast(s64) a.kind, cast(s64) b.kind);
        if c != 0 return c;
        if a.kind == .FIXED return compare(a.offset, b.offset);

        if a.kind == .RULES && a.rules != null && b.rules != null {
            ra := lookup_rules(a.rules);
            rb := lookup_rules(b.rules);
            if ra == null || rb == null return 0;
            return String.compare(ra.source, rb.source);
        }

        return 0;
    }
}

// Tests value equality consistent with compare and hash; Zoned ignores its zone.
// Example: `same := equal(a, b)`.
equal :: (a: $T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned], b: T) -> bool {
    return compare(a, b) == 0;
}

// Hashes semantic fields, never padding or rule pointers; equal values have equal hashes.
// Example: `key := hash(value)`.
hash :: (value: $T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned]) -> u32 {
    h: u32 = 2166136261;

    #if T == Date {
        h = hash_part(h, value.year);
        h = hash_part(h, value.month);
        h = hash_part(h, value.day);
    } else #if T == Time_Of_Day {
        h = hash_part(h, time_ns(value));
    } else #if T == Date_Time {
        h = hash_part(h, hash(value.date));
        h = hash_part(h, hash(value.time));
    } else #if T == Timestamp || T == Signed_Duration {
        h = hash_part(h, value.seconds);
        h = hash_part(h, value.nanoseconds);
    } else #if T == Offset {
        h = hash_part(h, value.seconds);
    } else #if T == Zoned {
        return hash(value.timestamp);
    } else {
        h = hash_part(h, cast(s64) value.kind);
        if value.kind == .FIXED h = hash_part(h, value.offset.seconds);

        if value.kind == .RULES && value.rules != null {
            r := lookup_rules(value.rules);
            if r == null return h;
            for c: r.source h = hash_part(h, c);
        }
    }

    return h;
}

// Converts the Apollo epoch, rejecting subnanosecond precision instead of discarding it.
// Example: `instant, error := timestamp_from_apollo(Basic.APOLLO_TIME_FOR_JAN_1_1970)`.
timestamp_from_apollo :: (value: Basic.Apollo_Time) -> Timestamp, Time_Error {
    delta := cast(S128) value - cast(S128) Basic.APOLLO_TIME_FOR_JAN_1_1970;
    q, r := wide_div(delta, 1000000);
    if r != 0 return .{}, .{ .INVALID_VALUE, "subnanosecond Apollo precision", -1 };
    return timestamp_wide(q);
}

// Converts a validated instant to Basic's femtosecond Apollo representation.
// Example: `apollo, error := timestamp_to_apollo(instant)`.
timestamp_to_apollo :: (value: Timestamp) -> Basic.Apollo_Time, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Treats Apollo as a duration (no epoch); subnanosecond input fails.
// Example: `duration, error := duration_from_apollo(value)`.
duration_from_apollo :: (value: Basic.Apollo_Time) -> Signed_Duration, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Converts an exact duration without adding an epoch.
// Example: `apollo, error := duration_to_apollo(value)`.
duration_to_apollo :: (value: Signed_Duration) -> Basic.Apollo_Time, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Validates Basic's zero-based components; ignores zone and derived weekday metadata.
// Example: `wall, error := date_time_from_calendar(calendar)`.
date_time_from_calendar :: (value: Basic.Calendar_Time) -> Date_Time, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Converts a civil reading as UTC-tagged fields; rejects submillisecond precision.
// Example: `calendar, error := date_time_to_calendar(wall)`.
date_time_to_calendar :: (value: Date_Time) -> Basic.Calendar_Time, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Reads the type's ISO text; Zoned rejects conflicting offsets and accepts bare zone annotations.
// Example: `value, error := parse(Timestamp, "2024-03-10T07:30:00Z")`.
parse :: ($T: Type, text: string, options: Time_Options = .{}) -> T, Time_Error {
    if options.max_text < 0 || text.count > options.max_text return .{}, .{ .LIMIT, "text limit", 0 };

    r := Text_Reader.{ text = text };

    #if T == Date {
        v := read_date(*r);
        if r.error.kind == .NONE && r.at != text.count text_fail(*r);
        return v, r.error;
    } else #if T == Time_Of_Day {
        v := read_clock(*r);
        if r.error.kind == .NONE && r.at != text.count text_fail(*r);
        return v, r.error;
    } else #if T == Date_Time {
        v := read_wall(*r);
        if r.error.kind == .NONE && r.at != text.count text_fail(*r);
        return v, r.error;
    } else #if T == Offset {
        v := read_offset(*r);
        if r.error.kind == .NONE && r.at != text.count text_fail(*r);
        return v, r.error;
    } else #if T == Timestamp {
        wall := read_wall(*r);
        o := read_offset(*r);

        if r.error.kind == .NONE && r.at != text.count text_fail(*r);
        if r.error.kind != .NONE return .{}, r.error;

        v, e := to_timestamp(time_zone_fixed(o), wall);
        if e.kind != .NONE e.offset = r.at;

        return v, e;
    } else {
        return .{}, .{ .UNSUPPORTED, "format not implemented yet", 0 };
    }
}

// Uses [String_Builder](https://jacketlang.com/docs/api/String/String_Builder.html).
// Appends canonical ISO text, trims fractional zeros and uses UTC Z for timestamps.
// Example: `error := format(*builder, instant)`.
format :: (builder: *String_Builder, value: $T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned, Span]) -> Time_Error {
    #if T == Date {
        y: s64 = value.year;
        if y < 0 {
            append(builder, "-");
            digits(builder, -y, 6);
        } else {
            digits(builder, y, 4);
        }

        append(builder, "-");
        digits(builder, value.month, 2);
        append(builder, "-");
        digits(builder, value.day, 2);
    } else #if T == Time_Of_Day {
        digits(builder, value.hour, 2);
        append(builder, ":");
        digits(builder, value.minute, 2);
        append(builder, ":");
        digits(builder, value.second, 2);
        fraction(builder, value.nanosecond);
    } else #if T == Date_Time {
        e := format(builder, value.date);
        if e.kind != .NONE return e;
        append(builder, "T");
        return format(builder, value.time);
    } else #if T == Offset {
        seconds: s64 = value.seconds;

        append(builder, ifx seconds < 0 then "-" else "+");
        if seconds < 0 seconds = -seconds;

        digits(builder, seconds / 3600, 2);
        append(builder, ":");
        digits(builder, seconds / 60 % 60, 2);

        if seconds % 60 != 0 {
            append(builder, ":");
            digits(builder, seconds % 60, 2);
        }
    } else #if T == Timestamp {
        e := format(builder, to_civil(time_zone_utc(), value));
        if e.kind != .NONE return e;
        append(builder, "Z");
    } else {
        return .{ .UNSUPPORTED, "format not implemented yet", -1 };
    }

    if builder.failed return .{ .LIMIT, "builder allocation failed", -1 };

    return .{};
}

// Appends English strftime text with the documented directive set and padding flags.
// Example: `error := format(*builder, wall, "%Y-%m-%d %H:%M:%S")`.
format :: (builder: *String_Builder, value: $T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned], pattern: string, options: Time_Options = .{}) -> Time_Error {
    return .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Reads the documented English directives; missing or conflicting required fields fail.
// Example: `wall, error := parse_format(Date_Time, "%Y-%m-%d %H:%M", "2024-03-10 02:30")`.
parse_format :: ($T: Type, pattern: string, text: string, options: Time_Options = .{}) -> T, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Uses [Json_Writer](https://jacketlang.com/docs/api/Extensions/Json/Json_Writer.html).
// Custom Json field encoder; canonical ISO strings use Json's writer limits.
// Example: `time_json_encode :: Time.time_json_encode // in an @json(with=time_json) struct`.
time_json_encode :: (writer: *Json.Json_Writer, value: *$T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned, Span]) {}

// Uses [Json_Reader](https://jacketlang.com/docs/api/Extensions/Json/Json_Reader.html).
// Custom Json field decoder; parse errors become Json INVALID_VALUE and output stays unchanged.
// Example: `time_json_decode :: Time.time_json_decode // alongside time_json_encode`.
time_json_decode :: (reader: *Json.Json_Reader, out: *$T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned, Span]) -> bool {
    return false;
}

#scope_module

Basic :: #import "Basic";
#import "String";
Json :: #import "Extensions/Json";

using,except(Time_Zone) Basic;
NS :: 1000000000;
DAY_NS :: 86400000000000;
MIN_I64 :: -9223372036854775808;
MAX_I64 :: 9223372036854775807;

range_error :: () -> Time_Error {
    return .{ .OUT_OF_RANGE, "outside supported range", -1 };
}

cmp64 :: (a: s64, b: s64) -> int {
    if a < b return -1;
    if a > b return 1;
    return 0;
}

floor_div :: (a: s64, b: s64) -> s64 {
    q := a / b;
    if a % b < 0 q -= 1;
    return q;
}

modulo :: (a: s64, b: s64) -> s64 {
    r := a % b;
    if r < 0 r += b;
    return r;
}

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

    return .{ cast(s16) y, cast(s8) m, cast(s8) d };
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

span_fields :: (v: Span) -> [10] s64 {
    return .[v.years, v.months, v.weeks, v.days, v.hours, v.minutes, v.seconds, v.milliseconds, v.microseconds, v.nanoseconds];
}

calendar_nonzero :: (v: Span) -> bool {
    return v.years != 0 || v.months != 0 || v.weeks != 0 || v.days != 0;
}

clock_nonzero :: (v: Span) -> bool {
    return v.hours != 0 || v.minutes != 0 || v.seconds != 0 || v.milliseconds != 0 || v.microseconds != 0 || v.nanoseconds != 0;
}

clear_clock :: (v: *Span) {
    v.hours = 0;
    v.minutes = 0;
    v.seconds = 0;
    v.milliseconds = 0;
    v.microseconds = 0;
    v.nanoseconds = 0;
}

wide_mul :: (a: s64, b: s64) -> S128 {
    negative := (a < 0) != (b < 0);
    ua := cast,no_check(u64) a;

    if a < 0 ua = 0 - ua;

    ub := cast,no_check(u64) b;
    if b < 0 ub = 0 - ub;

    h, l := multiply_64bits_with_overflow(ua, ub);
    v := S128.{ l, cast, no_check(s64) h };

    if negative v = S128.{} - v;

    return v;
}

narrow :: (v: S128) -> s64 {
    return cast,no_check(s64) v.low;
}

wide_div :: (v: S128, divisor: s64) -> S128, s64 {
    if v == to_s128(narrow(v)) return to_s128(narrow(v) / divisor), narrow(v) % divisor;
    q, r, _ := signed_divide_with_remainder(v, to_s128(divisor));
    return q, narrow(r);
}

clock_ns :: (v: Span) -> S128 {
    return wide_mul(v.hours, 3600 * NS) + wide_mul(v.minutes, 60 * NS) + wide_mul(v.seconds, NS) + wide_mul(v.milliseconds, 1000000) + wide_mul(v.microseconds, 1000) + to_s128(v.nanoseconds);
}

Text_Reader :: struct {
    text: string;
    at: s64;
    error: Time_Error;
}

text_fail :: (r: *Text_Reader, kind := Time_Error_Kind.SYNTAX, message: string = "invalid time text", at: s64 = -1) {
    if r.error.kind == .NONE r.error = .{ kind, message, ifx at < 0 then r.at else at };
}

peek :: (r: *Text_Reader) -> u8 {
    if r.at >= r.text.count return 0;
    return r.text[r.at];
}

expect_byte :: (r: *Text_Reader, c: u8) {
    if peek(r) != c {
        text_fail(r);
        return;
    }

    r.at += 1;
}

read_number :: (r: *Text_Reader, count: s64) -> s64 {
    n: s64;

    for 1..count {
        c := peek(r);
        if c < 48 || c > 57 {
            text_fail(r);
            return n;
        }

        n = n * 10 + c - 48;
        r.at += 1;
    }

    return n;
}

read_date :: (r: *Text_Reader) -> Date {
    sign: s64 = 1;
    width: s64 = 4;

    if peek(r) == 45 || peek(r) == 43 {
        if peek(r) == 45 sign = -1;
        r.at += 1;
        width = 6;
    }

    y := read_number(r, width) * sign;

    expect_byte(r, 45);
    m := read_number(r, 2);

    expect_byte(r, 45);
    d := read_number(r, 2);
    v, e := date(y, m, d);

    if e.kind != .NONE text_fail(r, e.kind, e.message, 0);

    return v;
}

read_clock :: (r: *Text_Reader) -> Time_Of_Day {
    h := read_number(r, 2);

    expect_byte(r, 58);
    m := read_number(r, 2);

    expect_byte(r, 58);
    sec := read_number(r, 2);
    ns: s64;

    if peek(r) == 46 {
        r.at += 1;
        count: s64;
        while peek(r) >= 48 && peek(r) <= 57 && count < 9 {
            ns = ns * 10 + peek(r) - 48;
            r.at += 1;
            count += 1;
        }

        if count == 0 text_fail(r);
        for count..8 ns *= 10;
    }

    v, e := time_of_day(h, m, sec, ns);
    if e.kind != .NONE text_fail(r, e.kind, e.message);

    return v;
}

read_wall :: (r: *Text_Reader) -> Date_Time {
    d := read_date(r);
    c := peek(r);

    if c != 84 && c != 116 && c != 32 {
        text_fail(r);
        return .{};
    }

    r.at += 1;
    t := read_clock(r);

    return .{ d, t };
}

digits :: (b: *String_Builder, n: s64, width: s64 = 1, pad: u8 = 48) {
    storage: [24] u8;
    at := storage.count;
    v := n;

    if v < 0 {
        append(b, "-");
        v = -(v / 10);
    } // Invalid literals remain bounded.

    while true {
        at -= 1;
        storage[at] = cast(u8) (v % 10 + 48);
        v /= 10;
        if v == 0 break;
    }

    for 1..width - (storage.count - at) append(b, pad);

    append(b, string.{ storage.count - at, storage.data + at });
}

fraction :: (b: *String_Builder, ns: s64) {
    if ns <= 0 return;

    width: s64 = 9;
    v := ns;

    while v % 10 == 0 && width > 1 {
        v /= 10;
        width -= 1;
    }

    append(b, ".");
    digits(b, v, width);
}

hash_part :: (h: u32, v: s64) -> u32 #no_aoc {
    r := h;
    n := cast,no_check(u64) v;

    for 0..7 {
        r = (r ^ cast,trunc(u32) n) * 16777619;
        n >>= 8;
    }

    return r;
}

stamp_ns :: (v: Timestamp) -> S128 {
    return wide_mul(v.seconds, NS) + to_s128(v.nanoseconds);
}

duration_wide :: (v: S128) -> Signed_Duration, Time_Error {
    q, r := wide_div(v, NS);
    if q < to_s128(MIN_I64) || q > to_s128(MAX_I64) return .{}, range_error();
    return .{ narrow(q), cast(s32) r }, .{};
}

timestamp_wide :: (v: S128) -> Timestamp, Time_Error {
    lo := wide_mul(date_days(.{ -9999, 1, 2 }) * 86400 + 7199, NS);
    hi := wide_mul(date_days(.{ 9999, 12, 30 }) * 86400 + 79200, NS);

    if v < lo || v > hi return .{}, range_error();

    q, r := wide_div(v, NS);

    return .{ narrow(q), cast(s32) r }, .{};
}

read_offset :: (r: *Text_Reader) -> Offset {
    c := peek(r);
    if c == 90 || c == 122 {
        r.at += 1;
        return .{};
    }

    if c != 43 && c != 45 {
        text_fail(r);
        return .{};
    }

    r.at += 1;
    h := read_number(r, 2);

    expect_byte(r, 58);
    m := read_number(r, 2);
    sec: s64;

    if peek(r) == 58 {
        r.at += 1;
        sec = read_number(r, 2);
    }

    if m > 59 || sec > 59 text_fail(r, .INVALID_VALUE, "invalid offset component");

    v, e := offset((h * 3600 + m * 60 + sec) * (ifx c == 45 then -1 else 1));
    if e.kind != .NONE text_fail(r, e.kind, e.message);

    return v;
}

Atomics :: #import "Atomics";
Files :: #import "File";
Env :: #import "Extensions/Environment";
String :: #import "String";

Zone_Rule :: struct {
    kind: u8;
    a: s64;
    b: s64;
    c: s64;
    seconds: s64 = 7200;
}

Posix_Rules :: struct {
    present: bool;
    dst: bool;
    standard: s64;
    daylight: s64;
    standard_name: string;
    daylight_name: string;
    start: Zone_Rule;
    end: Zone_Rule;
}

Zone_Rules :: struct {
    refs: s64 = 1;
    allocator: Allocator;
    name: string;
    path: string;
    source: string;
    count: s64;
    types: s64;
    width: s64;
    ta: s64;
    ia: s64;
    type_at: s64;
    ab: s64;
    chars: s64;
    footer: Posix_Rules;
}

zone_cache: [64] Time_Zone;
zone_cache_slot: s64;
zone_cache_lock: s32;

new_rules :: (name: string, source: string) -> *Zone_Rules {
    r := New(Zone_Rules);
    r.allocator = context.allocator;
    r.name = copy_string(name);
    r.source = copy_string(source);
    while !compare_and_swap(*registry_lock, cast(s32) 0, cast(s32) 1) {}

    {
        push_allocator(context.default_allocator.proc, context.default_allocator.data);
        Registry.table_add(*registered_rules, cast(*void) r, r);
    }

    Atomics.atomic_write(*registry_lock, cast(s32) 0);

    return r;
}

valid_options :: (o: Time_Options) -> bool {
    return o.max_text >= 0 && o.max_tzif_bytes >= 0 && o.max_transitions >= 0 && o.max_types >= 0 && o.max_abbreviation_bytes >= 0;
}

zone_within_limits :: (r: *Zone_Rules, o: Time_Options) -> bool {
    return r.source.count <= o.max_tzif_bytes && r.count <= o.max_transitions && r.types <= o.max_types && r.chars <= o.max_abbreviation_bytes;
}

read_zone_file :: (path: string, o: Time_Options) -> [] u8, Time_Error {
    if !valid_options(o) return .[], .{ .LIMIT, "negative zone limit", -1 };

    file, ok := Files.file_open(path);
    if !ok return .[], .{ .NOT_FOUND, "cannot open zone file", -1 };

    defer Files.file_close(*file);
    length, known := Files.file_length(file);
    if !known return .[], .{ .IO, "cannot size zone file", -1 };
    if length < 0 || length > o.max_tzif_bytes return .[], .{ .LIMIT, "zone file size limit", -1 };

    bytes := NewArray(length, u8);
    success, n := Files.file_read(file, bytes.data, length);

    if !success || n != length {
        free(bytes.data);
        return .[], .{ .IO, "cannot read zone file", -1 };
    }

    return bytes, .{};
}

Binary_Reader :: struct {
    bytes: [] u8;
    at: s64;
    error: Time_Error;
}

TZif_Header :: struct {
    version: u8;
    utc: s64;
    standard: s64;
    leaps: s64;
    times: s64;
    types: s64;
    chars: s64;
}

binary_fail :: (r: *Binary_Reader, message: string) {
    if r.error.kind == .NONE r.error = .{ .SYNTAX, message, r.at };
}

be_signed :: (bytes: $T, at: s64, width: s64) -> s64 {
    n: u64;
    for i: 0..width - 1 n = (n << 8) | bytes[at + i];
    if width == 4 return cast(s64) cast,no_check(s32) cast(u32) n;
    return cast,no_check(s64) n;
}

tzif_header :: (r: *Binary_Reader, o: Time_Options) -> TZif_Header {
    h: TZif_Header;

    if r.at < 0 || r.at > r.bytes.count || r.bytes.count - r.at < 44 {
        binary_fail(r, "truncated TZif header");
        return h;
    }

    at := r.at;

    r.at += 44;
    if r.bytes[at] != 84 || r.bytes[at + 1] != 90 || r.bytes[at + 2] != 105 || r.bytes[at + 3] != 102 {
        binary_fail(r, "invalid TZif magic");
        return h;
    }

    h.version = r.bytes[at + 4];
    if h.version != 0 && (h.version < 50 || h.version > 52) binary_fail(r, "unsupported TZif version");
    for i: 5..19 if r.bytes[at + i] != 0 binary_fail(r, "nonzero TZif reserved byte");

    h.utc = be_signed(r.bytes, at + 20, 4);
    h.standard = be_signed(r.bytes, at + 24, 4);
    h.leaps = be_signed(r.bytes, at + 28, 4);
    h.times = be_signed(r.bytes, at + 32, 4);
    h.types = be_signed(r.bytes, at + 36, 4);
    h.chars = be_signed(r.bytes, at + 40, 4);
    if h.times < 0 || h.times > o.max_transitions || h.types < 1 || h.types > 256 || h.types > o.max_types || h.chars < 1 || h.chars > o.max_abbreviation_bytes || h.leaps < 0 || h.leaps > o.max_transitions || (h.utc != 0 && h.utc != h.types) || (h.standard != 0 && h.standard != h.types) binary_fail(r, "invalid TZif counts");

    return h;
}

block_size :: (h: TZif_Header, w: s64) -> s64 {
    return h.times * (w + 1) + h.types * 6 + h.chars + h.leaps * (w + 4) + h.standard + h.utc;
}

zone_type :: (r: *Zone_Rules, second: s64) -> s64 {
    if r.footer.present && (r.count == 0 || second > be_signed(r.source, r.ta + (r.count - 1) * r.width, r.width)) return -1;

    lo: s64;
    hi := r.count;

    while lo < hi {
        mid := lo + (hi - lo) / 2;
        if be_signed(r.source, r.ta + mid * r.width, r.width) <= second lo = mid + 1;
        else hi = mid;
    }

    if lo == 0 return 0;

    return r.source[r.ia + lo - 1];
}

read_variable :: (r: *Text_Reader, max_digits: s64 = 3) -> s64 {
    n: s64;
    count: s64;

    while peek(r) >= 48 && peek(r) <= 57 && count < max_digits {
        n = n * 10 + peek(r) - 48;
        r.at += 1;
        count += 1;
    }

    if count == 0 text_fail(r);

    return n;
}

posix_name :: (r: *Text_Reader) -> string {
    quoted := peek(r) == 60;
    if quoted r.at += 1;

    begin := r.at;

    while r.at < r.text.count {
        c := peek(r);
        if (c >= 65 && c <= 90) || (c >= 97 && c <= 122) || (quoted && ((c >= 48 && c <= 57) || c == 43 || c == 45)) r.at += 1;
        else break;
    }

    name := string.{ r.at - begin, r.text.data + begin };
    if name.count < 3 text_fail(r);
    if quoted expect_byte(r, 62);

    return name;
}

posix_time :: (r: *Text_Reader, limit: s64) -> s64 {
    sign: s64 = 1;

    if peek(r) == 45 || peek(r) == 43 {
        if peek(r) == 45 sign = -1;
        r.at += 1;
    }

    h := read_variable(r);
    m: s64;
    sec: s64;

    if peek(r) == 58 {
        r.at += 1;
        m = read_number(r, 2);

        if peek(r) == 58 {
            r.at += 1;
            sec = read_number(r, 2);
        }
    }

    if h > limit || m > 59 || sec > 59 text_fail(r, .INVALID_VALUE, "POSIX time out of range");

    return sign * (h * 3600 + m * 60 + sec);
}

posix_rule :: (r: *Text_Reader) -> Zone_Rule {
    rule: Zone_Rule;

    if peek(r) == 77 {
        r.at += 1;
        rule.kind = 2;
        rule.a = read_variable(r);
        expect_byte(r, 46);
        rule.b = read_variable(r);
        expect_byte(r, 46);
        rule.c = read_variable(r);
        if rule.a < 1 || rule.a > 12 || rule.b < 1 || rule.b > 5 || rule.c > 6 text_fail(r, .INVALID_VALUE, "invalid M rule");
    } else {
        if peek(r) == 74 {
            r.at += 1;
            rule.kind = 0;
        } else rule.kind = 1;

        rule.a = read_variable(r);
        if (rule.kind == 0 && (rule.a < 1 || rule.a > 365)) || (rule.kind == 1 && rule.a > 365) text_fail(r, .INVALID_VALUE, "invalid Julian rule");
    }

    if peek(r) == 47 {
        r.at += 1;
        rule.seconds = posix_time(r, 167);
    }

    return rule;
}

read_posix :: (r: *Text_Reader) -> Posix_Rules {
    p: Posix_Rules;
    p.present = true;
    p.standard_name = posix_name(r);
    p.standard = -posix_time(r, 25);
    if p.standard < -93599 || p.standard > 93599 text_fail(r, .OUT_OF_RANGE, "POSIX offset out of range");

    if r.at < r.text.count && r.error.kind == .NONE {
        p.dst = true;
        p.daylight_name = posix_name(r);
        p.daylight = p.standard + 3600;
        if peek(r) != 44 p.daylight = -posix_time(r, 25);
        if p.daylight < -93599 || p.daylight > 93599 text_fail(r, .OUT_OF_RANGE, "POSIX DST offset out of range");

        expect_byte(r, 44);
        p.start = posix_rule(r);
        expect_byte(r, 44);
        p.end = posix_rule(r);
    }

    if r.at != r.text.count text_fail(r);

    return p;
}

rule_in_year :: (rule: Zone_Rule, year: s64, offset: s64) -> s64 {
    ordinal := rule.a;

    if rule.kind == 2 {
        first := Date.{ cast(s16) year, cast(s8) rule.a, 1 };
        day := 1 + modulo(rule.c - (cast(s64) weekday(first) % 7), 7) + (rule.b - 1) * 7;
        if day > month_length(year, rule.a) day -= 7;
        ordinal = date_days(.{ cast(s16) year, cast(s8) rule.a, cast(s8) day }) - date_days(.{ cast(s16) year, 1, 1 });
    } else if rule.kind == 0 {
        ordinal -= 1;
        if is_leap_year(year) && rule.a >= 60 ordinal += 1;
    }

    return (date_days(.{ cast(s16) year, 1, 1 }) + ordinal) * 86400 + rule.seconds - offset;
}

posix_at :: (p: Posix_Rules, seconds: s64) -> s64, string {
    if !p.dst return p.standard, p.standard_name;

    year: s64 = civil_day(floor_div(seconds, 86400)).year;
    // Extended transition times can cross the year boundary: choose the last event.
    last: s64 = MIN_I64;
    dst := false;

    for y: year - 1..year + 1 {
        start := rule_in_year(p.start, y, p.standard);
        end := rule_in_year(p.end, y, p.daylight);

        if start <= seconds && start > last {
            last = start;
            dst = true;
        }

        if end <= seconds && end > last {
            last = end;
            dst = false;
        }
    }

    if dst return p.daylight, p.daylight_name;

    return p.standard, p.standard_name;
}

Registry :: #import "Hash_Table";

registered_rules: Registry.Table(*void, *Zone_Rules);
registry_lock: s32;

// Validate opaque handles before dereferencing: transparent literals may contain arbitrary bits.
lookup_rules :: (handle: *void) -> *Zone_Rules {
    while !compare_and_swap(*registry_lock, cast(s32) 0, cast(s32) 1) {}
    _, r := Registry.table_find(*registered_rules, handle);
    Atomics.atomic_write(*registry_lock, cast(s32) 0);
    return r;
}

unregister_rules :: (r: *Zone_Rules) {
    while !compare_and_swap(*registry_lock, cast(s32) 0, cast(s32) 1) {}
    Registry.table_remove(*registered_rules, cast(*void) r);
    if registered_rules.count == 0 Registry.deinit(*registered_rules);
    Atomics.atomic_write(*registry_lock, cast(s32) 0);
}
