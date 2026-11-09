// Exact instants, durations and fixed offsets.

// Validates seconds east of UTC, including historical second offsets.
//
// ```jai
// o, error := offset(-14400);
// assert(error.kind == .NONE && o.seconds == -14400);
// _, bad := offset(100000);
// assert(bad.kind != .NONE);
// ```
offset :: (seconds: s64) -> Offset, Time_Error {
    if seconds < -93599 || seconds > 93599 return .{}, range_error();
    return .{ cast(s32) seconds }, .{};
}

// Normalizes signed seconds and nanoseconds, checking representability.
//
// ```jai
// d, error := signed_duration(1, -1);
// assert(error.kind == .NONE && d.seconds == 0 && d.nanoseconds == 999999999);
// ```
signed_duration :: (seconds: s64, nanoseconds: s64 = 0) -> Signed_Duration, Time_Error {
    return duration_wide(wide_mul(seconds, NS) + to_s128(nanoseconds));
}

// Reads Basic.current_time_consensus and converts its Apollo epoch; no new clock path.
//
// ```jai
// instant, error := timestamp_now();
// assert(error.kind == .NONE);
// ```
timestamp_now :: () -> Timestamp, Time_Error {
    return timestamp_from_apollo(Basic.current_time_consensus());
}

// Converts signed Unix seconds, checking the UTC year range.
//
// ```jai
// instant, error := timestamp_from_seconds(-1);
// assert(error.kind == .NONE && instant.seconds == -1);
// ```
timestamp_from_seconds :: (value: s64) -> Timestamp, Time_Error {
    return timestamp_wide(wide_mul(value, NS));
}

// Converts signed Unix milliseconds, checking the UTC year range.
//
// ```jai
// instant, error := timestamp_from_millis(1500);
// assert(error.kind == .NONE && instant.seconds == 1 && instant.nanoseconds == 500000000);
// ```
timestamp_from_millis :: (value: s64) -> Timestamp, Time_Error {
    return timestamp_wide(wide_mul(value / 1000, NS) + wide_mul(value % 1000, NS / 1000));
}

// Converts signed Unix microseconds, checking the UTC year range.
//
// ```jai
// instant, error := timestamp_from_micros(1500000);
// assert(error.kind == .NONE && instant.seconds == 1 && instant.nanoseconds == 500000000);
// ```
timestamp_from_micros :: (value: s64) -> Timestamp, Time_Error {
    return timestamp_wide(wide_mul(value / 1000000, NS) + wide_mul(value % 1000000, NS / 1000000));
}

// Converts signed Unix nanoseconds, checking the UTC year range.
//
// ```jai
// instant, error := timestamp_from_nanos(1500000000);
// assert(error.kind == .NONE && instant.seconds == 1 && instant.nanoseconds == 500000000);
// ```
timestamp_from_nanos :: (value: s64) -> Timestamp, Time_Error {
    return timestamp_wide(wide_mul(value / 1000000000, NS) + wide_mul(value % 1000000000, NS / 1000000000));
}

// Adds exact elapsed time with checked range and normalization.
//
// ```jai
// start, _ := timestamp_from_seconds(0);
// step, _ := signed_duration(90);
// later, error := add(start, step);
// assert(error.kind == .NONE && later.seconds == 90);
// ```
add :: (value: Timestamp, amount: Signed_Duration) -> Timestamp, Time_Error {
    return timestamp_wide(stamp_ns(value) + wide_mul(amount.seconds, NS) + to_s128(amount.nanoseconds));
}

// Adds clock units only; calendar units, including days, fail.
//
// ```jai
// start, _ := timestamp_from_seconds(0);
// later, error := add(start, Span.{ hours = 1 });
// assert(error.kind == .NONE && later.seconds == 3600);
// _, calendar := add(start, Span.{ days = 1 });
// assert(calendar.kind != .NONE);
// ```
add :: (value: Timestamp, amount: Span) -> Timestamp, Time_Error {
    d, e := duration_from_span(amount);
    if e.kind != .NONE return .{}, e;
    return timestamp_wide(stamp_ns(value) + wide_mul(d.seconds, NS) + to_s128(d.nanoseconds));
}

// Returns exact b minus a without calendar interpretation.
//
// ```jai
// a, _ := timestamp_from_seconds(10);
// b, _ := timestamp_from_seconds(25);
// elapsed := duration_between(a, b);
// assert(elapsed.seconds == 15);
// ```
duration_between :: (a: Timestamp, b: Timestamp) -> Signed_Duration {
    q, r := wide_div(stamp_ns(b) - stamp_ns(a), NS);
    return .{ narrow(q), cast(s32) r };
}

#scope_module

stamp_ns :: (v: Timestamp) -> S128 {
    return wide_mul(v.seconds, NS) + to_s128(v.nanoseconds);
}

duration_wide :: (v: S128) -> Signed_Duration, Time_Error {
    q, r := wide_div(v, NS);
    if q < to_s128(MIN_I64) || q > to_s128(MAX_I64) return .{}, range_error();
    return .{ narrow(q), cast(s32) r }, .{};
}

// Unix seconds of -9999-01-02T01:59:59 and 9999-12-30T22:00:00: the instants whose reading in any
// offset (+/-25:59:59) stays inside the civil range.
MIN_SECONDS :: -4371586 * 86400 + 7199;
MAX_SECONDS :: 2932895 * 86400 + 79200;

timestamp_wide :: (v: S128) -> Timestamp, Time_Error {
    lo := wide_mul(MIN_SECONDS, NS);
    hi := wide_mul(MAX_SECONDS, NS);

    if v < lo || v > hi return .{}, range_error();

    q, r := wide_div(v, NS);

    return .{ narrow(q), cast(s32) r }, .{};
}

raw_stamp :: (seconds: s64, ns: s64) -> Timestamp {
    q, r := wide_div(wide_mul(seconds, NS) + to_s128(ns), NS);
    return .{ narrow(q), cast(s32) r };
}
