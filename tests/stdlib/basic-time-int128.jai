// Clocks and sleeping (monotonic never goes back, a sleep lasts at least as long as asked, the
// consensus clock agrees with the calendar), Apollo_Time conversions, comparisons and calendar
// round trips, and the Int128 operators checked against each other on pseudo-random values.
#import "Basic";

main :: () {
    check_clocks();
    check_apollo_conversions();
    check_calendar_round_trips();
    check_int128();
    print("ok\n");
}

check_clocks :: () {
    // Monotonic time never goes backwards.
    previous := current_time_monotonic();
    assert(previous != APOLLO_TIME_INVALID);
    for 1..2000 {
        now := current_time_monotonic();
        assert(now >= previous, "monotonic time went back at sample %", it);
        previous = now;
    }
    seconds_a := get_time();
    seconds_b := get_time();
    assert(seconds_b >= seconds_a, "get_time went back: % then %", seconds_a, seconds_b);
    since := seconds_since_init();
    assert(since >= 0, "seconds_since_init %", since);
    assert(seconds_since_init() >= since);

    // A sleep lasts at least what was asked for, and not wildly longer.
    for milliseconds: s32.[1, 15, 40] {
        before := current_time_monotonic();
        sleep_milliseconds(milliseconds);
        slept := to_microseconds(current_time_monotonic() - before);
        // Windows' Sleep can return up to a clock tick early by the high-resolution clock.
        #if OS == .WINDOWS lower := milliseconds * 1000 - 2000;
        else lower := milliseconds * 1000;
        assert(slept >= lower, "sleep_milliseconds(%) took % us", milliseconds, slept);
        assert(slept < 5_000_000, "sleep_milliseconds(%) took % us", milliseconds, slept);
    }
    before := current_time_monotonic();
    sleep_milliseconds(0);
    sleep_milliseconds(-5);
    assert(to_milliseconds(current_time_monotonic() - before) < 1000, "non-positive sleeps return at once");

    // The consensus clock is a plausible date (the browser sandbox starts its clock in late 2023).
    now := current_time_consensus();
    assert(now != APOLLO_TIME_INVALID);
    calendar := to_calendar(now);
    assert(calendar.year >= 2023 && calendar.year < 2200, "consensus time says year %", calendar.year);
    assert(calendar.time_zone == .UTC);
    local := to_calendar(now, .LOCAL);
    assert(local.time_zone == .LOCAL);
    #if OS == .WASM {
        // WASM has no time zones: local time is UTC.
        assert(local.hour == calendar.hour && local.minute == calendar.minute && local.day_of_month_starting_at_0 == calendar.day_of_month_starting_at_0, "local % vs UTC %", calendar_to_string(local), calendar_to_string(calendar));
    }
    // A local calendar converts back to the same instant (to the millisecond).
    back := calendar_to_apollo(local);
    difference := to_milliseconds(back - now, DO_ROUNDING = true);
    assert(difference == 0, "local round trip is off by % ms", difference);
}

check_apollo_conversions :: () {
    one_second := seconds_to_apollo(1);
    assert(milliseconds_to_apollo(1000) == one_second);
    assert(microseconds_to_apollo(1_000_000) == one_second);
    assert(nanoseconds_to_apollo(1_000_000_000) == one_second);
    us, ok := to_microseconds(microseconds_to_apollo(-123_456_789));
    assert(ok && us == -123_456_789, "microseconds round trip: %", us);
    ns: s64;
    ns, ok = to_nanoseconds(microseconds_to_apollo(7) + nanoseconds_to_apollo(3));
    assert(ok && ns == 7003);
    // Truncation and rounding of microseconds.
    half := nanoseconds_to_apollo(1500);
    us, ok = to_microseconds(half);
    assert(ok && us == 1);
    us, ok = to_microseconds(half, DO_ROUNDING = true);
    assert(ok && us == 2);
    us, ok = to_microseconds(nanoseconds_to_apollo(-1499), DO_ROUNDING = true);
    assert(ok && us == -1, "rounding -1.499 us gave %", us);
    _, ok = to_microseconds(APOLLO_TIME_INVALID);
    assert(!ok);
    _, ok = to_microseconds(seconds_to_apollo(0x7fff_ffff_ffff_ffff));
    assert(!ok, "an out-of-range conversion fails");

    // Comparisons, including across the sign and across the 64-bit limb boundary.
    values := Apollo_Time.[seconds_to_apollo(-100), nanoseconds_to_apollo(-1), .{}, nanoseconds_to_apollo(1), seconds_to_apollo(20), seconds_to_apollo(40), APOLLO_TIME_FOR_JAN_1_1970];
    for a, i: values for b, j: values {
        assert((a < b) == (i < j) && (a <= b) == (i <= j) && (a > b) == (i > j) && (a >= b) == (i >= j), "comparing % and %", i, j);
        expected: s32 = ifx i < j then -1 else ifx i > j then 1 else 0;
        assert(compare_apollo_times(a, b) == expected, "compare_apollo_times(%, %)", i, j);
    }
    // Arithmetic.
    sum := seconds_to_apollo(20) + milliseconds_to_apollo(-500);
    assert(to_milliseconds(sum) == 19500);
    difference := seconds_to_apollo(1) - seconds_to_apollo(3);
    assert(to_seconds(difference) == -2 && difference < .{});
    assert(to_float64_seconds(milliseconds_to_apollo(-2500)) == -2.5);
    assert(to_apollo(-0.25) == milliseconds_to_apollo(-250));
    assert(to_apollo(0.0 / 0.0) == APOLLO_TIME_INVALID, "NaN is invalid");
    assert(to_apollo(1e300) == APOLLO_TIME_INVALID, "too large is invalid");
    _, valid := to_float64_seconds(APOLLO_TIME_INVALID);
    assert(!valid);
}

check_calendar_round_trips :: () {
    // Every day from 1899 to 2101 at noon: fields are consistent and convert back exactly.
    start := calendar_to_apollo(.{ year = 1899, month_starting_at_0 = 0, day_of_month_starting_at_0 = 0, hour = 12 });
    day := seconds_to_apollo(86400);
    t := start;
    previous_weekday: s8 = -1;
    days_in_month := 0;
    for 0..73800 {
        c := to_calendar(t);
        assert(c.hour == 12 && c.minute == 0 && c.second == 0 && c.millisecond == 0);
        assert(calendar_to_apollo(c) == t, "round trip failed on day %", it);
        if previous_weekday >= 0 assert(c.day_of_week_starting_at_0 == (previous_weekday + 1) % 7, "weekdays advance by one on day %", it);
        previous_weekday = c.day_of_week_starting_at_0;
        if c.day_of_month_starting_at_0 == 0 {
            if it > 0 {
                // The month that just ended had the right number of days.
                ended_month := (c.month_starting_at_0 + 11) % 12;
                ended_year := ifx c.month_starting_at_0 == 0 then c.year - 1 else c.year;
                assert(days_in_month == days_of(ended_year, ended_month), "%-% had % days", ended_year, ended_month + 1, days_in_month);
            }
            days_in_month = 0;
        }
        days_in_month += 1;
        t = t + day;
    }
    // Known dates.
    epoch := to_calendar(APOLLO_TIME_FOR_JAN_1_1970);
    assert(epoch.year == 1970 && epoch.month_starting_at_0 == 0 && epoch.day_of_month_starting_at_0 == 0 && epoch.day_of_week_starting_at_0 == 4, "epoch %", epoch);
    leap := Calendar_Time.{ year = 2024, month_starting_at_0 = 1, day_of_month_starting_at_0 = 28, hour = 23, minute = 59, second = 59, millisecond = 999 };
    leap_time := calendar_to_apollo(leap);
    after := to_calendar(leap_time + milliseconds_to_apollo(1));
    assert(after.year == 2024 && after.month_starting_at_0 == 2 && after.day_of_month_starting_at_0 == 0 && after.hour == 0, "after 2024-02-29T23:59:59.999: %", calendar_to_iso_string(after));
    // Out-of-range fields are normalized.
    overflow := Calendar_Time.{ year = 2023, month_starting_at_0 = 13, day_of_month_starting_at_0 = 0 };
    normalized := to_calendar(calendar_to_apollo(overflow));
    assert(normalized.year == 2024 && normalized.month_starting_at_0 == 1, "month 14 of 2023 is %", calendar_to_string(normalized));
    negative_month := Calendar_Time.{ year = 2023, month_starting_at_0 = -1 };
    assert(to_calendar(calendar_to_apollo(negative_month)).year == 2022);
    // Millisecond rounding when converting to a calendar.
    rounded := to_calendar(APOLLO_TIME_FOR_JAN_1_1970 + microseconds_to_apollo(1500));
    assert(rounded.millisecond == 2, "1.5 ms rounds to %", rounded.millisecond);
    before_epoch := to_calendar(APOLLO_TIME_FOR_JAN_1_1970 - milliseconds_to_apollo(1));
    assert(before_epoch.year == 1969 && before_epoch.month_starting_at_0 == 11 && before_epoch.day_of_month_starting_at_0 == 30 && before_epoch.millisecond == 999, "%", calendar_to_iso_string(before_epoch));
    assert(calendar_to_iso_string(before_epoch) == "1969-12-31T23:59:59.999Z");
    invalid := to_calendar(APOLLO_TIME_INVALID);
    assert(invalid.year == 0 && invalid.month_starting_at_0 == 0);
}

days_of :: (year: s32, month: s32) -> s64 {
    lengths :: s64.[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if month == 1 && ((year % 4 == 0 && year % 100 != 0) || year % 400 == 0) return 29;
    return lengths[month];
}

state: u64 = 0x9e37_79b9_7f4a_7c15;

next :: () -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    return state;
}

check_int128 :: () {
    // Comparisons agree with 64-bit arithmetic for values that fit.
    samples := s64.[-9223372036854775807 - 1, -5, -1, 0, 1, 2, 77, 9223372036854775807];
    for a: samples for b : samples {
        sa, sb := to_s128(a), to_s128(b);
        assert((sa < sb) == (a < b) && (sa <= sb) == (a <= b) && (sa > sb) == (a > b) && (sa >= sb) == (a >= b), "S128 comparing % and %", a, b);
        ua, ub := to_u128(cast,no_check(u64) a), to_u128(cast,no_check(u64) b);
        x, y := cast,no_check(u64) a, cast,no_check(u64) b;
        assert((ua < ub) == (x < y) && (ua <= ub) == (x <= y) && (ua > ub) == (x > y) && (ua >= ub) == (x >= y), "U128 comparing % and %", x, y);
        if b != 0 && !(a == -9223372036854775807 - 1 && b == -1) {
            assert(sa / sb == to_s128(a / b), "S128 % / %", a, b);
            assert(sa / b == to_s128(a / b), "S128 % / s64 %", a, b);
        }
        if y != 0 assert(ua / ub == to_u128(x / y), "U128 % / %", x, y);
    }
    // Comparisons where only the high limb decides.
    assert(U128.{ 0, 1 } > U128.{ 0xffff_ffff_ffff_ffff, 0 });
    assert(S128.{ 0, -1 } < S128.{ 0, 0 } && S128.{ 0xffff_ffff_ffff_ffff, -1 } <= S128.{ 0, 0 });

    // Shifts: a left shift then a right shift by the same amount keeps the bits that stayed.
    for 1..300 {
        value := U128.{ next(), next() };
        shift := cast(u8) (next() % 128);
        left := value << shift;
        back := left >> shift;
        mask := (U128.{ 0xffff_ffff_ffff_ffff, 0xffff_ffff_ffff_ffff } << shift) >> shift;
        assert(back == U128.{ value.low & mask.low, value.high & mask.high }, "shift % of %", shift, value);
        signed := to_s128(value);
        assert(to_u128(signed << shift) == left, "S128 << agrees with U128 <<");
        // An arithmetic right shift equals division rounding toward minus infinity.
        arithmetic := signed >> shift;
        if signed.high >= 0 assert(to_u128(arithmetic) == value >> shift, "S128 >> of a positive value");
    }
    // Division: quotient * divisor + remainder gives the dividend back, remainder < divisor.
    for 1..300 {
        u := U128.{ next(), next() };
        v := U128.{ next(), next() >> cast(u8) (next() % 64) };
        if it % 3 == 0 v.high = 0;
        if !v continue;
        q, r, error := unsigned_divide_with_remainder(u, v);
        assert(!error);
        assert(r < v, "remainder not below the divisor");
        assert(q * v + r == u, "q*v + r != u for % / %", u, v);
        assert(u / v == q);
        su := to_s128(u);
        sv := to_s128(v);
        if !(sv == to_s128(0)) {
            sq, sr, serror := signed_divide_with_remainder(su, sv);
            if !serror {
                assert(sq * sv + sr == su, "signed q*v + r != u");
                assert(su / sv == sq);
                // The remainder takes the dividend's sign.
                if !(sr == to_s128(0)) assert((sr.high < 0) == (su.high < 0), "remainder sign");
            }
        }
    }
    // Division by a 64-bit factor.
    big := S128.{ 0, 1 };
    assert(big / 2 == S128.{ 0x8000_0000_0000_0000, 0 });
    assert(-big / 4 == S128.{ 0xc000_0000_0000_0000, -1 });
}
