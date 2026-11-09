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
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Validates a clock reading; leap seconds are rejected.
// Example: `t, error := time_of_day(23, 59, 59, 1)`.
time_of_day :: (hour: s64, minute: s64, second: s64, nanosecond: s64 = 0) -> Time_Of_Day, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Validates and combines civil components.
// Example: `dt, error := date_time(day, clock)`.
date_time :: (day: Date, clock: Time_Of_Day) -> Date_Time {
    return .{};
}

// Validates a civil date and clock reading together.
// Example: `dt, error := date_time(2024, 3, 10, 2, 30)`.
date_time :: (year: s64, month: s64, day: s64, hour: s64 = 0, minute: s64 = 0, second: s64 = 0, nanosecond: s64 = 0) -> Date_Time, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Validates seconds east of UTC, including historical second offsets.
// Example: `o, error := offset(-14400)`.
offset :: (seconds: s64) -> Offset, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Normalizes signed seconds and nanoseconds, checking representability.
// Example: `d, error := signed_duration(1, -1)`.
signed_duration :: (seconds: s64, nanoseconds: s64 = 0) -> Signed_Duration, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Validates the single-sign invariant without balancing units.
// Example: `s, error := span_check(.{months = 1, days = 2})`.
span_check :: (value: Span) -> Span, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Returns the Gregorian month length after validating year and month.
// Example: `n, error := days_in_month(2024, 2)`.
days_in_month :: (year: s64, month: s64) -> s64, Time_Error {
    return 0, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Tests the Gregorian leap rule within the supported year range.
// Example: `leap := is_leap_year(2000)`.
is_leap_year :: (year: s64) -> bool {
    return false;
}

// Returns the ISO weekday after validating the date.
// Example: `day := weekday(value)`.
weekday :: (value: Date) -> Weekday {
    return .{};
}

// Returns the ordinal day, starting at one.
// Example: `ordinal := day_of_year(value)`.
day_of_year :: (value: Date) -> s64 {
    return 0;
}

// Returns ISO week-year and week number; week one contains January 4.
// Example: `year, week := iso_week(value)`.
iso_week :: (value: Date) -> s64, s64 {
    return 0, 0;
}

// Validates and converts an ISO week date.
// Example: `d, error := date_from_iso_week(2024, 1, .MONDAY)`.
date_from_iso_week :: (year: s64, week: s64, day: Weekday) -> Date {
    return .{};
}

// Returns the first day of the validated date's month.
// Example: `d := first_of_month(value)`.
first_of_month :: (value: Date) -> Date {
    return .{};
}

// Returns the last day of the validated date's month.
// Example: `d := last_of_month(value)`.
last_of_month :: (value: Date) -> Date {
    return .{};
}

// Finds occurrence 1..5 or -1..-5 from the end; missing occurrences fail.
// Example: `d, error := nth_weekday_of_month(2024, 3, .SUNDAY, 2)`.
nth_weekday_of_month :: (year: s64, month: s64, day: Weekday, nth: s64) -> Date, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Applies years/months with month-end policy, then weeks/days; clock units fail.
// Example: `d, error := add(value, Span.{months = 1}, .REJECT)`.
add :: (value: Date, amount: Span, overflow := Overflow.CONSTRAIN) -> Date, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Applies calendar units before exact clock units, carrying across midnight.
// Example: `dt, error := add(value, Span.{days = 1, hours = 2})`.
add :: (value: Date_Time, amount: Span, overflow := Overflow.CONSTRAIN) -> Date_Time, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Adds clock units modulo one day; all calendar units fail.
// Example: `t, error := add(value, Span.{hours = 25})`.
add :: (value: Time_Of_Day, amount: Span) -> Time_Of_Day, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Measures b minus a; validates units, positive increments and range.
// Example: `s, error := until(a, b)`.
until :: (a: Date, b: Date, largest := Unit.DAY, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1) -> Span, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
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
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
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
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Reverses every field of a validated span.
// Example: `s := span_negate(value)`.
span_negate :: (value: Span) -> Span {
    return .{};
}

// Returns a nonnegative span; span_check rejects signed minima.
// Example: `s := span_abs(value)`.
span_abs :: (value: Span) -> Span {
    return .{};
}

// Reads Basic.current_time_consensus and converts its Apollo epoch; no new clock path.
// Example: `instant, error := timestamp_now()`.
timestamp_now :: () -> Timestamp, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Converts signed Unix seconds, checking the UTC year range.
// Example: `instant, error := timestamp_from_seconds(-1)`.
timestamp_from_seconds :: (value: s64) -> Timestamp, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Converts signed Unix milliseconds, checking the UTC year range.
// Example: `instant, error := timestamp_from_millis(-1)`.
timestamp_from_millis :: (value: s64) -> Timestamp, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Converts signed Unix microseconds, checking the UTC year range.
// Example: `instant, error := timestamp_from_micros(-1)`.
timestamp_from_micros :: (value: s64) -> Timestamp, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Converts signed Unix nanoseconds, checking the UTC year range.
// Example: `instant, error := timestamp_from_nanos(-1)`.
timestamp_from_nanos :: (value: s64) -> Timestamp, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Adds exact elapsed time with checked range and normalization.
// Example: `instant, error := add(value, duration)`.
add :: (value: Timestamp, amount: Signed_Duration) -> Timestamp, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Adds clock units only; calendar units, including days, fail.
// Example: `instant, error := add(value, Span.{hours = 24})`.
add :: (value: Timestamp, amount: Span) -> Timestamp, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Returns exact b minus a without calendar interpretation.
// Example: `elapsed := duration_between(a, b)`.
duration_between :: (a: Timestamp, b: Timestamp) -> Signed_Duration {
    return .{};
}

// Returns the allocation-free UTC zone.
// Example: `zone := time_zone_utc()`.
time_zone_utc :: () -> Time_Zone {
    return .{};
}

// Validates and constructs an allocation-free fixed-offset zone.
// Example: `zone := time_zone_fixed(value)`.
time_zone_fixed :: (value: Offset) -> Time_Zone {
    return .{};
}

// Loads and caches a safe IANA name; directory overrides TZDIR for deterministic fixtures.
// Example: `zone, error := time_zone_get("America/New_York")`.
time_zone_get :: (name: string, options: Time_Options = .{}, directory: string = "") -> Time_Zone, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Uses TZ, otherwise /etc/localtime; Windows without TZ returns UNSUPPORTED.
// Example: `zone, error := time_zone_system()`.
time_zone_system :: (options: Time_Options = .{}) -> Time_Zone, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Copies validated TZif v1-v4 rules and POSIX footer within bounded storage.
// Example: `zone, error := time_zone_from_tzif("Example/Zone", bytes)`.
time_zone_from_tzif :: (name: string, bytes: [] u8, options: Time_Options = .{}) -> Time_Zone, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Parses POSIX TZ rules including M/J/Julian forms and extended transition times.
// Example: `zone, error := time_zone_from_posix("EST5EDT,M3.2.0/2,M11.1.0/2")`.
time_zone_from_posix :: (text: string, options: Time_Options = .{}) -> Time_Zone, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Acquires an independent owner; UTC and fixed zones need no reference count.
// Example: `owned := time_zone_retain(borrowed)`.
time_zone_retain :: (value: Time_Zone) -> Time_Zone {
    return .{};
}

// Releases one owned reference and resets the handle to UTC; borrowed copies become invalid.
// Example: `time_zone_free(*zone)`.
time_zone_free :: (value: *Time_Zone) {}

// Resolves the offset at an exact instant, using footer rules beyond stored transitions.
// Example: `o := offset_at(zone, instant)`.
offset_at :: (zone: Time_Zone, instant: Timestamp) -> Offset {
    return .{};
}

// Returns a zone-borrowed abbreviation for formatting and conformance.
// Example: `text := abbreviation_at(zone, instant)`.
abbreviation_at :: (zone: Time_Zone, instant: Timestamp) -> string {
    return .{};
}

// Converts an instant in an explicit zone, rejecting civil range overflow.
// Example: `wall := to_civil(zone, instant)`.
to_civil :: (zone: Time_Zone, instant: Timestamp) -> Date_Time {
    return .{};
}

// Resolves gaps/folds using the requested policy and checks UTC range.
// Example: `instant, error := to_timestamp(zone, wall, .REJECT)`.
to_timestamp :: (zone: Time_Zone, wall: Date_Time, disambiguation := Disambiguation.COMPATIBLE) -> Timestamp, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Starts a borrowed cursor strictly after instant (strictly before when reverse).
// Example: `cursor := transitions(zone, instant)`.
transitions :: (zone: Time_Zone, instant: Timestamp, reverse := false) -> Time_Transitions {
    return .{};
}

// Advances a transition cursor; false with no error means the range is exhausted.
// Example: `change, found, error := next(*cursor)`.
next :: (cursor: *Time_Transitions) -> Time_Transition, bool, Time_Error {
    return .{}, false, .{ .UNSUPPORTED, "Time API declaration only", -1 };
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
    return 0;
}

// Tests value equality consistent with compare and hash; Zoned ignores its zone.
// Example: `same := equal(a, b)`.
equal :: (a: $T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned], b: T) -> bool {
    return false;
}

// Hashes semantic fields, never padding or rule pointers; equal values have equal hashes.
// Example: `key := hash(value)`.
hash :: (value: $T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned]) -> u32 {
    return 0;
}

// Converts the Apollo epoch, rejecting subnanosecond precision instead of discarding it.
// Example: `instant, error := timestamp_from_apollo(Basic.APOLLO_TIME_FOR_JAN_1_1970)`.
timestamp_from_apollo :: (value: Basic.Apollo_Time) -> Timestamp, Time_Error {
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
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
    return .{}, .{ .UNSUPPORTED, "Time API declaration only", -1 };
}

// Uses [String_Builder](https://jacketlang.com/docs/api/String/String_Builder.html).
// Appends canonical ISO text, trims fractional zeros and uses UTC Z for timestamps.
// Example: `error := format(*builder, instant)`.
format :: (builder: *String_Builder, value: $T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned, Span]) -> Time_Error {
    return .{ .UNSUPPORTED, "Time API declaration only", -1 };
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
