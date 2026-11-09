// The value, error and option types.

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

// A constant-space cursor borrowing a zone; e.g. `cursor := transitions(zone, instant)`.
Time_Transitions :: struct {
    zone: Time_Zone;
    cursor: Timestamp;
    reverse: bool;
    exhausted: bool;
}
