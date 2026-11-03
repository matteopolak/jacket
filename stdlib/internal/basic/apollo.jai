// A point or span of time, stored as a 128-bit count of femtoseconds. A time of day counts from 20 July 1969, 20:17 UTC,
// not from 1970; use `APOLLO_TIME_FOR_JAN_1_1970` for the Unix zero. It does not overflow in practice and keeps
// sub-nanosecond precision. Make one with `seconds_to_apollo` and its siblings or `current_time_consensus`, and read it
// back with `to_seconds` and the like. `APOLLO_TIME_INVALID` marks a failure.
//
// ```jai
// a := milliseconds_to_apollo(1500);
// b := seconds_to_apollo(2);
//
// assert(compare_apollo_times(a, b) < 0);
// seconds, ok := to_seconds(b);
//
// assert(ok && seconds == 2);
// ```
Apollo_Time :: #type,isa S128;
// The `Apollo_Time` that functions return when they fail. Compare with `compare_apollo_times`.
APOLLO_TIME_INVALID :: Apollo_Time.{ 0, 0x8000000000000000 };
// The `Apollo_Time` of 1970-01-01 00:00 UTC, the zero of the Unix clock.
//
// ```jai
// ct := to_calendar(APOLLO_TIME_FOR_JAN_1_1970);
// assert(ct.year == 1970 && ct.month_starting_at_0 == 0 && ct.day_of_month_starting_at_0 == 0);
// ```
APOLLO_TIME_FOR_JAN_1_1970 :: Apollo_Time.{ low = 0xdbd87f0edb7c8000, high = 0x300 };
// Femtoseconds in a second.
FEMTOSECONDS :: 1000000000000000;

// Which clock `to_calendar` uses: `.UTC`, or `.LOCAL` for the computer's own time zone.
Time_Zone :: enum u32 {
    UTC :: 0;
    LOCAL :: 1;
}

// A time broken into year, month, day and time of day. Months, days of the month and days of the week count from 0, so
// January is 0 and Sunday is 0. Make one with `to_calendar`.
//
// ```jai
// ct := to_calendar(APOLLO_TIME_FOR_JAN_1_1970);
// assert(ct.year == 1970 && ct.day_of_week_starting_at_0 == 4); // a Thursday
// ```
Calendar_Time :: struct {
    year: s32;
    month_starting_at_0: s8;
    day_of_month_starting_at_0: s8;
    day_of_week_starting_at_0: s8;
    hour: s8;
    minute: s8;
    second: s8;
    millisecond: s16;
    time_zone: Time_Zone;
}

// Scales `m` by `FACTOR` femtoseconds. The building block of the `*_to_apollo` procedures.
ConvertToApollo :: (m: s64, FACTOR: s64) #expand {
    `return scale_signed_duration(m, FACTOR);
}

// An `Apollo_Time` of `m` seconds.
//
// ```jai
// t := seconds_to_apollo(90);
// minutes, ok := to_milliseconds(t);
// assert(ok && minutes == 90000);
// ```
seconds_to_apollo :: (m: s64) -> Apollo_Time {
    return scale_duration(m, 1000000000000000);
}

// An `Apollo_Time` of `m` milliseconds.
//
// ```jai
// assert(to_microseconds(milliseconds_to_apollo(3)) == 3000);
// ```
milliseconds_to_apollo :: (m: s64) -> Apollo_Time {
    return scale_duration(m, 1000000000000);
}

// An `Apollo_Time` of `m` microseconds.
//
// ```jai
// assert(to_nanoseconds(microseconds_to_apollo(5)) == 5000);
// ```
microseconds_to_apollo :: (m: s64) -> Apollo_Time {
    return scale_duration(m, 1000000000);
}

// An `Apollo_Time` of `m` nanoseconds.
//
// ```jai
// assert(to_nanoseconds(nanoseconds_to_apollo(42)) == 42);
// ```
nanoseconds_to_apollo :: (m: s64) -> Apollo_Time {
    return scale_duration(m, 1000000);
}

// Divides `t` by `divisor` femtoseconds. The building block of the `to_seconds` family.
ConvertFromApollo :: inline (t: Apollo_Time, divisor: u64, $DO_ROUNDING: bool) -> (result: s64, success: bool) #no_aoc {
    value := U128.{ t.low, cast,no_check(u64) t.high };
    negative := t.high < 0;

    if negative {
        value.low = 0 - value.low;
        value.high = ~value.high;
        if value.low == 0 value.high += 1;
    }

    quotient, remainder, error := unsigned_divide_with_remainder(value, U128.{ divisor, 0 });
    if error return 0, false;

    #if DO_ROUNDING {
        if remainder.low >= divisor / 2 + divisor % 2 {
            quotient.low += 1;
            if quotient.low == 0 quotient.high += 1;
        }
    }

    limit: u64 = 0x7fffffffffffffff;
    if negative limit += 1;
    if quotient.high != 0 || quotient.low > limit return 0, false;
    if negative quotient.low = 0 - quotient.low;

    return cast,no_check(s64) quotient.low, true;
}

// `t` in whole seconds. The second result is `false` when the value does not fit an `s64`. Truncates toward zero, or rounds
// to nearest with `DO_ROUNDING`.
//
// ```jai
// seconds, ok := to_seconds(milliseconds_to_apollo(2500));
// assert(ok && seconds == 2);
// rounded, ok2 := to_seconds(milliseconds_to_apollo(2500), true);
// assert(ok2 && rounded == 3);
// ```
to_seconds :: (t: Apollo_Time, $DO_ROUNDING := false) -> (result: s64, success: bool) {
    return ConvertFromApollo(t, 1000000000000000, DO_ROUNDING);
}

// `t` in whole milliseconds; see `to_seconds` for the rounding and the success flag.
//
// ```jai
// ms, ok := to_milliseconds(seconds_to_apollo(3));
// assert(ok && ms == 3000);
// ```
to_milliseconds :: (t: Apollo_Time, $DO_ROUNDING := false) -> (result: s64, success: bool) {
    return ConvertFromApollo(t, 1000000000000, DO_ROUNDING);
}

// `t` in whole microseconds; see `to_seconds` for the rounding and the success flag.
//
// ```jai
// us, ok := to_microseconds(milliseconds_to_apollo(1));
// assert(ok && us == 1000);
// ```
to_microseconds :: (t: Apollo_Time, $DO_ROUNDING := false) -> (result: s64, success: bool) {
    return ConvertFromApollo(t, 1000000000, DO_ROUNDING);
}

// `t` in whole nanoseconds; see `to_seconds` for the rounding and the success flag.
//
// ```jai
// ns, ok := to_nanoseconds(microseconds_to_apollo(1));
// assert(ok && ns == 1000);
// ```
to_nanoseconds :: (t: Apollo_Time, $DO_ROUNDING := false) -> (result: s64, success: bool) {
    return ConvertFromApollo(t, 1000000, DO_ROUNDING);
}

// Returns -1, 0 or 1 when `a` is before, equal to or after `b`.
//
// ```jai
// assert(compare_apollo_times(seconds_to_apollo(1), seconds_to_apollo(2)) == -1);
// assert(compare_apollo_times(seconds_to_apollo(2), seconds_to_apollo(2)) == 0);
// ```
compare_apollo_times :: (a: Apollo_Time, b: Apollo_Time) -> s32 {
    if a.high < b.high return -1;
    if a.high > b.high return 1;
    if a.low < b.low return -1;
    if a.low > b.low return 1;

    return 0;
}

// `input` as seconds in a `float64`. The second result is `false` for `APOLLO_TIME_INVALID` or a value too big.
//
// ```jai
// seconds, ok := to_float64_seconds(milliseconds_to_apollo(1500));
// assert(ok && seconds == 1.5);
// ```
to_float64_seconds :: (input: Apollo_Time) -> (result: float64, success: bool) {
    if input.low == APOLLO_TIME_INVALID.low && input.high == APOLLO_TIME_INVALID.high return 0, false;
    whole, valid := to_seconds(input);
    if !valid return 0, false;
    return seconds_with_fraction(input, whole), true;
}

// An `Apollo_Time` of `seconds`, a `float64`. Infinity and NaN give `APOLLO_TIME_INVALID`.
//
// ```jai
// t := to_apollo(0.25);
// assert(to_milliseconds(t) == 250);
// ```
to_apollo :: (seconds: float64) -> Apollo_Time #no_aoc {
    // Scale the binary significand exactly before truncating sub-femtosecond bits.
    // This avoids overflowing an s64 intermediate for large finite durations.
    bits: u64;

    memcpy(*bits, *seconds, size_of(float64));
    encoded_exponent := (bits >> 52) & 0x7ff;
    if encoded_exponent == 0x7ff return APOLLO_TIME_INVALID;
    if encoded_exponent == 0 return .{};

    significand := (bits & 0xfffffffffffff) | 0x10000000000000;
    high, low := multiply_64bits_with_overflow(significand, FEMTOSECONDS);
    MANTISSA_BITS :: 52;
    shift := cast(s64) encoded_exponent - 1023 - MANTISSA_BITS;

    while shift > 0 {
        if (high & 0x4000000000000000) != 0 return APOLLO_TIME_INVALID;
        high = (high << 1) | (low >> 63);
        low <<= 1;
        shift -= 1;
    }

    if shift <= -128 return .{};

    while shift < 0 {
        low = (low >> 1) | (high << 63);
        high >>= 1;
        shift += 1;
    }

    if (bits >> 63) != 0 {
        low = 0 - low;
        high = ~high;
        if low == 0 high += 1;
    }

    return .{ low, cast,no_check(s64) high };
}

#scope_file
seconds_with_fraction :: (input: Apollo_Time, whole: s64) -> float64 #no_aoc {
    integral := seconds_to_apollo(whole);
    // The signed residual has magnitude below FEMTOSECONDS. Its low word
    // suffices after the checked whole-second conversion above.
    residual := input.low - integral.low;
    negative := input.high < 0;

    if negative residual = 0 - residual;

    fraction := cast(float64) residual / cast(float64) FEMTOSECONDS;
    if negative return cast(float64) whole - fraction;

    return cast(float64) whole + fraction;
}

scale_duration :: (value: s64, factor: u64) -> Apollo_Time #no_aoc {
    high, low := multiply_64bits_with_overflow(cast,no_check(u64) value, factor);
    if value < 0 high -= factor;
    return Apollo_Time.{ low, cast,no_check(s64) high };
}

scale_signed_duration :: (value: s64, factor: s64) -> Apollo_Time #no_aoc {
    high, low := multiply_64bits_with_overflow(cast,no_check(u64) value, cast,no_check(u64) factor);
    if value < 0 high -= cast,no_check(u64) factor;
    if factor < 0 high -= cast,no_check(u64) value;
    return Apollo_Time.{ low, cast,no_check(s64) high };
}
