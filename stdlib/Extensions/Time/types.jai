// The value, error and option types.

// A Gregorian date in years -9999..9999; construct with `date(2024, 3, 10)`.
//
// ```jai
// day := Date.{ 2024, 3, 10 };
// assert(day.month == 3);
// assert(Date.{}.year == 1970); // the default is 1970-01-01
// ```
Date :: struct {
    year: s16 = 1970;
    month: s8 = 1;
    day: s8 = 1;
}

// A clock reading without a zone.
//
// ```jai
// t, error := time_of_day(2, 30, 0);
// assert(error.kind == .NONE && t.hour == 2 && t.minute == 30);
// ```
Time_Of_Day :: struct {
    hour: s8;
    minute: s8;
    second: s8;
    nanosecond: s32;
}

// A wall-clock value, not an instant.
//
// ```jai
// wall, error := date_time(2024, 3, 10, 2, 30, 0);
// assert(error.kind == .NONE && wall.date.day == 10 && wall.time.hour == 2);
// ```
Date_Time :: struct {
    date: Date;
    time: Time_Of_Day;
}

// Unix seconds and a signed subsecond remainder.
// The remainder is in -999999999..999999999 and has the seconds' sign when seconds != 0.
//
// ```jai
// instant, error := timestamp_from_millis(1500);
// assert(error.kind == .NONE && instant.seconds == 1 && instant.nanoseconds == 500000000);
// ```
Timestamp :: struct {
    seconds: s64;
    nanoseconds: s32;
}

// Exact elapsed time, with the same remainder invariant as Timestamp.
//
// ```jai
// d, error := signed_duration(1, 5);
// assert(error.kind == .NONE && d.seconds == 1 && d.nanoseconds == 5);
// ```
Signed_Duration :: struct {
    seconds: s64;
    nanoseconds: s32;
}

// Seconds east of UTC, within +/-25:59:59.
//
// ```jai
// o, error := offset(19800);
// assert(error.kind == .NONE && o.seconds == 19800);
// ```
Offset :: struct {
    seconds: s32;
}

// The representation of a zone.
//
// ```jai
// assert(time_zone_utc().kind == .UTC);
// o, _ := offset(3600);
// assert(time_zone_fixed(o).kind == .FIXED);
// ```
Time_Zone_Kind :: enum u8 {
    UTC;
    FIXED;
    RULES;
}

// An immutable zone handle.
// RULES handles own one reference. Assignment borrows; time_zone_retain creates another owner.
// Release each owner once with time_zone_free; a Zoned borrows its zone.
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// assert(zone.kind == .RULES);
// ```
Time_Zone :: struct {
    kind: Time_Zone_Kind;
    offset: Offset;
    rules: *void; // opaque; only this module may dereference it
}

// An instant and a borrowed zone, with its resolved offset.
//
// ```jai
// instant, _ := timestamp_from_seconds(0);
// z := zoned(instant, time_zone_utc());
// assert(z.timestamp.seconds == 0 && z.offset.seconds == 0);
// ```
Zoned :: struct {
    timestamp: Timestamp;
    zone: Time_Zone;
    offset: Offset;
}

// Separate calendar and clock units, all zero or sharing one sign.
// Raw field assignment does not validate: use span_check before passing caller-built values to arithmetic.
//
// ```jai
// span, error := span_check(.{ months = 1, days = 2 });
// assert(error.kind == .NONE && span.months == 1 && span.days == 2);
// ```
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

// Units ordered from finest to coarsest.
//
// ```jai
// span, _ := span_round(.{ hours = 1, minutes = 40 }, .HOUR, .HOUR);
// assert(span.hours == 2);
// ```
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
//
// ```jai
// assert(weekday(Date.{ 2024, 3, 11 }) == .MONDAY);
// assert(cast(int) Weekday.SUNDAY == 7);
// ```
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
//
// ```jai
// half_up, _ := span_round(.{ seconds = 2, milliseconds = 500 }, .SECOND, .SECOND, .HALF_EXPAND);
// half_even, _ := span_round(.{ seconds = 2, milliseconds = 500 }, .SECOND, .SECOND, .HALF_EVEN);
// assert(half_up.seconds == 3 && half_even.seconds == 2);
// ```
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
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// gap := Date_Time.{ .{ 2024, 3, 10 }, .{ 2, 30, 0, 0 } };
// earlier, _ := to_timestamp(zone, gap, .EARLIER);
// later, _ := to_timestamp(zone, gap, .LATER);
// assert(later.seconds - earlier.seconds == 3600);
// _, rejected := to_timestamp(zone, gap, .REJECT);
// assert(rejected.kind == .GAP);
// ```
Disambiguation :: enum u8 {
    COMPATIBLE;
    EARLIER;
    LATER;
    REJECT;
}

// Month-end handling; e.g. January 31 plus one month constrains to February's last day.
//
// ```jai
// clamped, _ := add(Date.{ 2024, 1, 31 }, Span.{ months = 1 }, .CONSTRAIN);
// assert(clamped.day == 29);
// _, rejected := add(Date.{ 2024, 1, 31 }, Span.{ months = 1 }, .REJECT);
// assert(rejected.kind != .NONE);
// ```
Overflow :: enum u8 {
    CONSTRAIN;
    REJECT;
}

// Failure classification; e.g. `error.kind == .NONE` means success.
//
// ```jai
// _, error := date(2024, 13, 1);
// assert(error.kind == .INVALID_VALUE);
// _, parse_error := parse(Date, "nope");
// assert(parse_error.kind == .SYNTAX);
// ```
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

// Static message and zero-based parse byte offset (-1 for other errors).
// No error owns memory. Failed results must not be used.
//
// ```jai
// _, error := parse(Date, "2024/02-29");
// assert(error.kind == .SYNTAX && error.offset == 4);
// _, invalid := date(2023, 2, 29);
// assert(invalid.offset == -1); // only parse errors carry an offset
// ```
Time_Error :: struct {
    kind: Time_Error_Kind;
    message: string;
    offset: s64 = -1;
}

// Bounds on text, TZif data and rule counts.
// Each public parser checks these before copying or allocating input-derived storage.
//
// ```jai
// options := Time_Options.{ max_text = 8 };
// _, error := parse(Timestamp, "2024-03-10T07:30:00Z", options);
// assert(error.kind == .LIMIT);
// ```
Time_Options :: struct {
    max_text: s64 = 4096;
    max_tzif_bytes: s64 = 16 * 1024 * 1024;
    max_transitions: s64 = 1000000;
    max_types: s64 = 256;
    max_abbreviation_bytes: s64 = 65536;
}

// One offset change.
// Abbreviation slices borrow the iterator's zone owner.
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// start, _ := timestamp_from_seconds(1704067200);
// cursor := transitions(zone, start);
// change, _, _ := next(*cursor);
// assert(change.after.seconds - change.before.seconds == 3600);
// assert(change.abbreviation_after == "EDT");
// ```
Time_Transition :: struct {
    timestamp: Timestamp;
    before: Offset;
    after: Offset;
    abbreviation_before: string;
    abbreviation_after: string;
}

// A constant-space cursor borrowing a zone.
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// start, _ := timestamp_from_seconds(1704067200);
// cursor := transitions(zone, start);
// _, found, error := next(*cursor);
// assert(error.kind == .NONE && found);
// ```
Time_Transitions :: struct {
    zone: Time_Zone;
    cursor: Timestamp;
    reverse: bool;
    exhausted: bool;
}
