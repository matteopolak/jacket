// Jai types for Postgres values that have no exact Jai counterpart.

// A value that may be SQL NULL. `null` is true for NULL, and `value` is then zero.
Nullable :: struct(T: Type) {
    value: T;
    null: bool;
}

// The value, or `default` when it is NULL.
or_else :: (n: Nullable($T), default: T) -> T {
    if n.null return default;
    return n.value;
}

// A non-null `Nullable` holding `value`.
some :: (value: $T) -> Nullable(T) {
    return .{ value = value };
}

// A `json` or `jsonb` document holding a `T`. `get` decodes it with the Json module's decoder for
// `T`, under the result's `json_options` and the query's `json` (json_options.jai); as a parameter
// it is encoded with `write_json`. `OPTIONS` is a `Json_Columns` entry's options (packed by
// `pack_json_options`), which the generated schema puts in the type; 0 says nothing.
// `write` lets a parameter passed as `Any` find its encoder, so leave it as it is. `Json(Json_Raw)`
// keeps the document undecoded: it is what a generated schema gives a json column nobody mapped.
Json :: struct(T: Type, OPTIONS: u32 = 0) {
    value: T;
    write: (value: *T, builder: *String_Builder) = (value: *T, builder: *String_Builder) {
        // Json(Json_Raw) is a column with no mapped type: its text goes out as it is.
        #if T == Json_Raw append(builder, value.text);
        else Json_Module.write_json(builder, value.*);
    };
}

// `value` as a `json`/`jsonb` parameter.
json :: (value: $T) -> Json(T) {
    return .{ value = value };
}

// A `json` or `jsonb` document, undecoded: its text, which points into the result like any string.
// `decode_json` turns it into a `T` later. Also a parameter, sent as it is.
Json_Raw :: struct {
    text: string;
}

decode_json :: ($T: Type, raw: Json_Raw, options := Json_Options.{}) -> T, Json_Module.Json_Error {
    value, error := Json_Module.from_json(T, raw.text, module_json_options(.{ connection = options }, 0));
    return value, error;
}

// The document as a `Json_Value` tree, which the caller releases with `free_json` (from
// Extensions/Json); any JSON can be read this way, whatever `T` a column would map it to.
json_value :: (raw: Json_Raw, options := Json_Options.{}) -> Json_Module.Json_Value, Json_Module.Json_Error {
    value, error := Json_Module.from_json(Json_Module.Json_Value, raw.text, module_json_options(.{ connection = options }, 0));
    return value, error;
}

// `uuid`: the 16 bytes in network order.
Uuid :: struct {
    bytes: [16] u8;
}

// `timestamptz`: an instant, in microseconds since 2000-01-01 00:00:00 UTC (Postgres' own epoch).
// TIMESTAMP_INFINITY and TIMESTAMP_MINUS_INFINITY are `infinity` and `-infinity`.
Timestamp :: struct {
    microseconds: s64;
}

// `timestamp` (without time zone): a wall-clock reading, in microseconds since 2000-01-01 00:00:00.
Local_Timestamp :: struct {
    microseconds: s64;
}

// `date`: days since 2000-01-01. DATE_INFINITY and DATE_MINUS_INFINITY are `infinity` and
// `-infinity`.
Date :: struct {
    days: s32;
}

// `time` (without time zone): microseconds since midnight.
Time :: struct {
    microseconds: s64;
}

// `interval`: months, days and microseconds are kept apart, as Postgres keeps them, because a
// month has no fixed number of days and a day has no fixed number of microseconds across a
// daylight-saving change. From PostgreSQL 17 `infinity` is every field at S32_MAX / S64_MAX and
// `-infinity` every field at the minimum.
Interval :: struct {
    months: s32;
    days: s32;
    microseconds: s64;
}

// `numeric`: an exact decimal, as its text: "-12.3400", "0", "NaN", "Infinity", "-Infinity".
// `numeric_to_s64` and `numeric_to_float64` convert it.
Numeric :: struct {
    text: string;
}

// `infinity` and `-infinity` of timestamps and dates, as the server stores them.
TIMESTAMP_INFINITY :: S64_MAX;
TIMESTAMP_MINUS_INFINITY :: S64_MIN;
DATE_INFINITY :: S32_MAX;
DATE_MINUS_INFINITY :: S32_MIN;

MICROSECONDS_PER_SECOND :: 1_000_000;
MICROSECONDS_PER_MINUTE :: 60 * MICROSECONDS_PER_SECOND;
MICROSECONDS_PER_HOUR :: 60 * MICROSECONDS_PER_MINUTE;
MICROSECONDS_PER_DAY :: 86_400_000_000;

// Microseconds from 1970-01-01 (the Unix epoch) to 2000-01-01.
UNIX_TO_POSTGRES_MICROSECONDS :: 946_684_800_000_000;

timestamp_from_unix_microseconds :: (microseconds: s64) -> Timestamp {
    return .{ microseconds - UNIX_TO_POSTGRES_MICROSECONDS };
}

unix_microseconds :: (t: Timestamp) -> s64 {
    return t.microseconds + UNIX_TO_POSTGRES_MICROSECONDS;
}

// The current time from the system clock.
now :: () -> Timestamp {
    return timestamp_from_unix_microseconds(to_microseconds(current_time_consensus() - APOLLO_TIME_FOR_JAN_1_1970));
}

// The date of a proleptic Gregorian calendar day. Year 0 is 1 BC, as in ISO 8601.
date_from_civil :: (year: s64, month: s64, day: s64) -> Date {
    return .{ cast(s32) (days_from_civil(year, month, day) - DAYS_1970_TO_2000) };
}

civil_from_date :: (date: Date) -> year: s64, month: s64, day: s64 {
    year, month, day := civil_from_days(cast(s64) date.days + DAYS_1970_TO_2000);
    return year, month, day;
}

timestamp_from_civil :: (year: s64, month: s64, day: s64, hour := 0, minute := 0, second := 0, microsecond := 0) -> Timestamp {
    days := cast(s64) date_from_civil(year, month, day).days;
    return .{ days * MICROSECONDS_PER_DAY + ((hour * 60 + minute) * 60 + second) * 1_000_000 + microsecond };
}

// Spans of time as an `Interval`, for arithmetic on timestamps: `Pg.now() - Pg.days(7)`. A day is
// a calendar day, not 24 hours, as in Postgres; the rest are exact.
days :: (count: s32) -> Interval {
    return .{ days = count };
}

hours :: (count: s64) -> Interval {
    return .{ microseconds = count * MICROSECONDS_PER_HOUR };
}

minutes :: (count: s64) -> Interval {
    return .{ microseconds = count * MICROSECONDS_PER_MINUTE };
}

seconds :: (count: s64) -> Interval {
    return .{ microseconds = count * MICROSECONDS_PER_SECOND };
}

operator - :: (interval: Interval) -> Interval {
    return .{ -interval.months, -interval.days, -interval.microseconds };
}

operator + :: (a: Interval, b: Interval) -> Interval {
    return .{ a.months + b.months, a.days + b.days, a.microseconds + b.microseconds };
}

operator - :: (a: Interval, b: Interval) -> Interval {
    return .{ a.months - b.months, a.days - b.days, a.microseconds - b.microseconds };
}

// The months come first, clamping the day to the end of the month ("Jan 31 + 1 month" is the end of
// February), then the days and the rest, all on the UTC calendar. An infinite timestamp stays as it is.
operator + :: (t: Timestamp, interval: Interval) -> Timestamp {
    if t.microseconds == TIMESTAMP_INFINITY || t.microseconds == TIMESTAMP_MINUS_INFINITY return t;

    microseconds := t.microseconds;

    if interval.months != 0 {
        whole_days := floor_divide(microseconds, MICROSECONDS_PER_DAY);
        time_of_day := microseconds - whole_days * MICROSECONDS_PER_DAY;
        year, month, day := civil_from_date(.{ cast(s32) whole_days });
        index := year * MONTHS_PER_YEAR + (month - 1) + interval.months;

        year = floor_divide(index, MONTHS_PER_YEAR);
        month = index - year * MONTHS_PER_YEAR + 1;
        day = min(day, days_in_month(year, month));
        microseconds = cast(s64) date_from_civil(year, month, day).days * MICROSECONDS_PER_DAY + time_of_day;
    }

    return .{ microseconds + interval.days * MICROSECONDS_PER_DAY + interval.microseconds };
}

operator + :: (interval: Interval, t: Timestamp) -> Timestamp {
    return t + interval;
}

operator - :: (t: Timestamp, interval: Interval) -> Timestamp {
    return t + (-interval);
}

// `Uuid` from "0f8fad5b-d9cb-469f-a165-70867728950e" (dashes optional, any case).
parse_uuid :: (text: string) -> Uuid, bool {
    u: Uuid;
    count := 0;
    i := 0;

    while i < text.count {
        if text[i] == #char "-" {
            i += 1;
            continue;
        }

        if count == 16 || i + 1 >= text.count return .{}, false;

        high := hex_value(text[i]);
        low := hex_value(text[i + 1]);

        if high < 0 || low < 0 return .{}, false;

        u.bytes[count] = cast(u8) (high * 16 + low);
        count += 1;
        i += 2;
    }

    return u, count == 16;
}

// "0f8fad5b-d9cb-469f-a165-70867728950e", in temporary storage.
uuid_text :: (u: Uuid) -> string {
    DIGITS :: "0123456789abcdef";
    out := talloc_string(36);
    at := 0;

    DASH_BEFORE :: s64.[4, 6, 8, 10]; // 8-4-4-4-12 hex digits

    for u.bytes {
        if array_find(DASH_BEFORE, it_index) {
            out[at] = #char "-";
            at += 1;
        }

        out[at] = DIGITS[it >> 4];
        out[at + 1] = DIGITS[it & 0xf];
        at += 2;
    }

    return out;
}

// The integer part of a numeric; false for NaN, the infinities, or one out of range. A fraction
// is truncated toward zero.
numeric_to_s64 :: (n: Numeric) -> s64, bool {
    text := n.text;
    dot := find_index_from_left(text, #char ".");

    if dot >= 0 text.count = dot;
    if text == "-" || text == "-0" return 0, true;

    value, valid, rest := string_to_int(text, 10, s64);

    return value, valid && !rest;
}

numeric_to_float64 :: (n: Numeric) -> float64, bool {
    if n.text == "NaN" return FLOAT64_NAN, true;
    if n.text == "Infinity" return FLOAT64_INFINITY, true;
    if n.text == "-Infinity" return -FLOAT64_INFINITY, true;

    value, valid, rest := string_to_float64(n.text);

    return value, valid && !rest;
}

#scope_module

DAYS_1970_TO_2000 :: 10_957;

// The Gregorian calendar repeats every 400 years (an era) of 146,097 days. Counting years from
// March 1 puts the leap day last, so months 0..11 (March..February) have the lengths
// 31 30 31 30 31 31 30 31 30 31 31 28/29, and month m starts on day (153m + 2) / 5 of that year.
YEARS_PER_ERA :: 400;
YEARS_PER_CENTURY :: 100;
YEARS_PER_LEAP_DAY :: 4;
DAYS_PER_ERA :: 146_097;
DAYS_PER_YEAR :: 365;
DAYS_PER_4_YEARS :: 4 * DAYS_PER_YEAR + 1;
DAYS_PER_CENTURY :: 100 * DAYS_PER_YEAR + 24;
MONTHS_PER_YEAR :: 12;
MARCH :: 3;
MONTH_START_SCALE :: 153; // with MONTH_START_DIVISOR: the start of month m is (153m + 2) / 5
MONTH_START_DIVISOR :: 5;
DAYS_0000_03_01_TO_1970 :: 719_468; // from the March 1 that starts era 0 to 1970-01-01

// Days since 1970-01-01 of a proleptic Gregorian date (valid for any year).
days_from_civil :: (year: s64, month: s64, day: s64) -> s64 {
    y := ifx month < MARCH then year - 1 else year;
    era := floor_divide(y, YEARS_PER_ERA);
    year_of_era := y - era * YEARS_PER_ERA;
    shifted_month := (month + MONTHS_PER_YEAR - MARCH) % MONTHS_PER_YEAR;
    day_of_year := (MONTH_START_SCALE * shifted_month + 2) / MONTH_START_DIVISOR + day - 1;
    day_of_era := year_of_era * DAYS_PER_YEAR + year_of_era / 4 - year_of_era / 100 + day_of_year;

    return era * DAYS_PER_ERA + day_of_era - DAYS_0000_03_01_TO_1970;
}

civil_from_days :: (days: s64) -> year: s64, month: s64, day: s64 {
    z := days + DAYS_0000_03_01_TO_1970;
    era := floor_divide(z, DAYS_PER_ERA);
    day_of_era := z - era * DAYS_PER_ERA;
    // Leap days pushed back out, so that dividing by 365 gives the year of the era.
    year_of_era := (day_of_era - day_of_era / (DAYS_PER_4_YEARS - 1) + day_of_era / DAYS_PER_CENTURY - day_of_era / (DAYS_PER_ERA - 1)) / DAYS_PER_YEAR;
    day_of_year := day_of_era - (DAYS_PER_YEAR * year_of_era + year_of_era / 4 - year_of_era / 100);
    shifted_month := (MONTH_START_DIVISOR * day_of_year + 2) / MONTH_START_SCALE;
    day := day_of_year - (MONTH_START_SCALE * shifted_month + 2) / MONTH_START_DIVISOR + 1;
    month := (shifted_month + MARCH - 1) % MONTHS_PER_YEAR + 1;
    year := year_of_era + era * YEARS_PER_ERA;

    if month < MARCH year += 1;

    return year, month, day;
}

floor_divide :: (a: s64, b: s64) -> s64 {
    q := a / b;
    if (a % b != 0) && ((a < 0) != (b < 0)) q -= 1;
    return q;
}

DAYS_IN_MONTH :: s64.[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

days_in_month :: (year: s64, month: s64) -> s64 {
    leap := year % YEARS_PER_LEAP_DAY == 0 && (year % YEARS_PER_CENTURY != 0 || year % YEARS_PER_ERA == 0);

    return DAYS_IN_MONTH[month - 1] + ifx month == 2 && leap then 1 else 0;
}

hex_value :: (c: u8) -> s64 {
    if c >= #char "0" && c <= #char "9" return c - #char "0";
    if c >= #char "a" && c <= #char "f" return c - #char "a" + 10;
    if c >= #char "A" && c <= #char "F" return c - #char "A" + 10;
    return -1;
}
