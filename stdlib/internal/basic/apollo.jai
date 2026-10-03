Apollo_Time :: #type,isa S128;
APOLLO_TIME_INVALID :: Apollo_Time.{ 0, 0x8000000000000000 };
APOLLO_TIME_FOR_JAN_1_1970 :: Apollo_Time.{ low = 0xdbd87f0edb7c8000, high = 0x300 };
FEMTOSECONDS :: 1000000000000000;

Time_Zone :: enum u32 {
    UTC :: 0;
    LOCAL :: 1;
}

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

ConvertToApollo :: (m: s64, FACTOR: s64) #expand {
    `return scale_signed_duration(m, FACTOR);
}

seconds_to_apollo :: (m: s64) -> Apollo_Time {
    return scale_duration(m, 1000000000000000);
}

milliseconds_to_apollo :: (m: s64) -> Apollo_Time {
    return scale_duration(m, 1000000000000);
}

microseconds_to_apollo :: (m: s64) -> Apollo_Time {
    return scale_duration(m, 1000000000);
}

nanoseconds_to_apollo :: (m: s64) -> Apollo_Time {
    return scale_duration(m, 1000000);
}

ConvertFromApollo :: inline (t: Apollo_Time, divisor: u64, $DO_ROUNDING: bool) -> (result: s64, success: bool) #no_aoc {
    value := U128.{ t.low, cast(u64) t.high };
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

    return cast(s64) quotient.low, true;
}

to_seconds :: (t: Apollo_Time, $DO_ROUNDING := false) -> (result: s64, success: bool) {
    return ConvertFromApollo(t, 1000000000000000, DO_ROUNDING);
}

to_milliseconds :: (t: Apollo_Time, $DO_ROUNDING := false) -> (result: s64, success: bool) {
    return ConvertFromApollo(t, 1000000000000, DO_ROUNDING);
}

to_microseconds :: (t: Apollo_Time, $DO_ROUNDING := false) -> (result: s64, success: bool) {
    return ConvertFromApollo(t, 1000000000, DO_ROUNDING);
}

to_nanoseconds :: (t: Apollo_Time, $DO_ROUNDING := false) -> (result: s64, success: bool) {
    return ConvertFromApollo(t, 1000000, DO_ROUNDING);
}

compare_apollo_times :: (a: Apollo_Time, b: Apollo_Time) -> s32 {
    if a.high < b.high return -1;
    if a.high > b.high return 1;
    if a.low < b.low return -1;
    if a.low > b.low return 1;

    return 0;
}

to_float64_seconds :: (input: Apollo_Time) -> (result: float64, success: bool) {
    if input.low == APOLLO_TIME_INVALID.low && input.high == APOLLO_TIME_INVALID.high return 0, false;
    whole, valid := to_seconds(input);
    if !valid return 0, false;
    return seconds_with_fraction(input, whole), true;
}

// Scale the binary significand exactly before truncating sub-femtosecond bits.
// This avoids overflowing an s64 intermediate for large finite durations.
to_apollo :: (seconds: float64) -> Apollo_Time #no_aoc {
    bits: u64;

    memcpy(*bits, *seconds, size_of(float64));
    encoded_exponent := (bits >> 52) & 0x7ff;
    if encoded_exponent == 0x7ff return APOLLO_TIME_INVALID;
    if encoded_exponent == 0 return .{};

    significand := (bits & 0xfffffffffffff) | 0x10000000000000;
    high, low := multiply_64bits_with_overflow(significand, FEMTOSECONDS);
    shift := cast(s64) encoded_exponent - 1023 - 52;

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

    return .{ low, cast(s64) high };
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
    high, low := multiply_64bits_with_overflow(cast(u64) value, factor);
    if value < 0 high -= factor;
    return Apollo_Time.{ low, cast(s64) high };
}

scale_signed_duration :: (value: s64, factor: s64) -> Apollo_Time #no_aoc {
    high, low := multiply_64bits_with_overflow(cast(u64) value, cast(u64) factor);
    if value < 0 high -= cast(u64) factor;
    if factor < 0 high -= cast(u64) value;
    return Apollo_Time.{ low, cast(s64) high };
}
