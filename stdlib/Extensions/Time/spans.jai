// Calendar spans: differences, rounding, balancing and totals.

// Validates the single-sign invariant without balancing units. Example: `s, error := span_check(.{months = 1, days = 2})`.
span_check :: (value: Span) -> Span, Time_Error {
    sign: s64;

    for field, i: span_fields(value) {
        if field == MIN_I64 || (i < 4 && field == -2147483648) return .{}, .{ .OUT_OF_RANGE, "span field cannot be negated", -1 };

        current := cmp64(field, 0);
        if current != 0 {
            if sign != 0 && current != sign return .{}, .{ .INVALID_VALUE, "mixed span signs", -1 };
            sign = current;
        }
    }

    return value, .{};
}

// Measures b minus a; validates units, positive increments and range. Example: `s, error := until(a, b)`.
until :: (a: Date, b: Date, largest := Unit.DAY, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1) -> Span, Time_Error {
    return difference(a, b, largest, smallest, rounding, increment);
}

// Measures b minus a; validates units, positive increments and range. Example: `s, error := until(a, b)`.
until :: (a: Date_Time, b: Date_Time, largest := Unit.DAY, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1) -> Span, Time_Error {
    return difference(a, b, largest, smallest, rounding, increment);
}

// Measures b minus a; validates units, positive increments and range. Example: `s, error := until(a, b)`.
until :: (a: Timestamp, b: Timestamp, largest := Unit.SECOND, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1) -> Span, Time_Error {
    return difference(a, b, largest, smallest, rounding, increment);
}

// Measures b minus a; calendar units use the receiver's zone and real day lengths. Example: `s, error := until(a, b)`.
until :: (a: Zoned, b: Zoned, largest := Unit.DAY, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1, disambiguation := Disambiguation.COMPATIBLE) -> Span, Time_Error {
    return difference(a, b, largest, smallest, rounding, increment, disambiguation);
}

// Measures a minus b; validates units, positive increments and range. Example: `s, error := since(a, b)`.
since :: (a: Date, b: Date, largest := Unit.DAY, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1) -> Span, Time_Error {
    return difference(b, a, largest, smallest, rounding, increment);
}

// Measures a minus b; validates units, positive increments and range. Example: `s, error := since(a, b)`.
since :: (a: Date_Time, b: Date_Time, largest := Unit.DAY, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1) -> Span, Time_Error {
    return difference(b, a, largest, smallest, rounding, increment);
}

// Measures a minus b; validates units, positive increments and range. Example: `s, error := since(a, b)`.
since :: (a: Timestamp, b: Timestamp, largest := Unit.SECOND, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1) -> Span, Time_Error {
    return difference(b, a, largest, smallest, rounding, increment);
}

// Measures a minus b; calendar units use the receiver's zone and real day lengths. Example: `s, error := since(a, b)`.
since :: (a: Zoned, b: Zoned, largest := Unit.DAY, smallest := Unit.NANOSECOND, rounding := Rounding.TRUNC, increment: s64 = 1, disambiguation := Disambiguation.COMPATIBLE) -> Span, Time_Error {
    return difference(b, a, largest, smallest, rounding, increment, disambiguation);
}

// Balances to largest; calendar conversions require a relative value. Example: `value, error := span_balance(value, .DAY, relative = day)`.
span_balance :: (value: Span, largest: Unit) -> Span, Time_Error {
    if largest > .DAY return .{}, .{ .RELATIVE_REQUIRED, "calendar balancing requires relative value", -1 };
    if value.years != 0 || value.months != 0 || value.weeks != 0 return .{}, .{ .RELATIVE_REQUIRED, "calendar balancing requires relative value", -1 };
    return split_time(clock_ns(value) + wide_mul(value.days, DAY_NS), largest);
}

// Uses a relative Date; e.g. `span_balance(value, .DAY, origin)`.
span_balance :: (value: Span, largest: Unit, relative: Date) -> Span, Time_Error {
    return balance_relative(value, largest, Date_Time.{ relative, .{} });
}

// Uses a relative Zoned; e.g. `span_balance(value, .DAY, origin)`.
span_balance :: (value: Span, largest: Unit, relative: Zoned) -> Span, Time_Error {
    return balance_relative(value, largest, relative);
}

// Rounds to a positive increment, then balances; calendar rounding needs a relative value. Example: `value, error := span_round(value, .MINUTE, .HOUR)`.
span_round :: (value: Span, smallest: Unit, largest: Unit, rounding := Rounding.HALF_EXPAND, increment: s64 = 1) -> Span, Time_Error {
    if smallest > .DAY || largest > .DAY return .{}, .{ .RELATIVE_REQUIRED, "calendar rounding requires relative value", -1 };
    if increment < 1 || smallest > largest return .{}, .{ .INVALID_VALUE, "invalid rounding units or increment", -1 };
    if value.years != 0 || value.months != 0 || value.weeks != 0 return .{}, .{ .RELATIVE_REQUIRED, "calendar rounding requires relative value", -1 };

    n := clock_ns(value) + wide_mul(value.days, DAY_NS);
    rounded := round_wide(n, wide_mul(unit_scale(smallest), increment), rounding);

    return split_time(rounded, largest);
}

// Uses a relative Date; e.g. `span_round(value, .MINUTE, .HOUR, relative = origin)`.
span_round :: (value: Span, smallest: Unit, largest: Unit, rounding := Rounding.HALF_EXPAND, increment: s64 = 1, relative: Date) -> Span, Time_Error {
    return round_relative(value, smallest, largest, rounding, increment, Date_Time.{ relative, .{} });
}

// Uses a relative Zoned; e.g. `span_round(value, .MINUTE, .HOUR, relative = origin)`.
span_round :: (value: Span, smallest: Unit, largest: Unit, rounding := Rounding.HALF_EXPAND, increment: s64 = 1, relative: Zoned) -> Span, Time_Error {
    return round_relative(value, smallest, largest, rounding, increment, relative);
}

// Returns a possibly fractional total; zoned days follow actual day lengths. Example: `value, error := span_total(value, .HOUR, relative = z)`.
span_total :: (value: Span, unit: Unit) -> float64, Time_Error {
    if unit > .DAY return 0, .{ .RELATIVE_REQUIRED, "calendar total requires relative value", -1 };
    if value.years != 0 || value.months != 0 || value.weeks != 0 return 0, .{ .RELATIVE_REQUIRED, "calendar total requires relative value", -1 };
    return wide_float(clock_ns(value) + wide_mul(value.days, DAY_NS)) / cast(float64) unit_scale(unit), .{};
}

// Uses a relative Date; e.g. `span_total(value, .HOUR, origin)`.
span_total :: (value: Span, unit: Unit, relative: Date) -> float64, Time_Error {
    return total_relative(value, unit, Date_Time.{ relative, .{} });
}

// Uses a relative Zoned; e.g. `span_total(value, .HOUR, origin)`.
span_total :: (value: Span, unit: Unit, relative: Zoned) -> float64, Time_Error {
    return total_relative(value, unit, relative);
}

// Returns -1/0/1 by elapsed length; calendar units require a shared relative value. Example: `value, error := span_compare(a, b, relative = day)`.
span_compare :: (a: Span, b: Span) -> int, Time_Error {
    if calendar_nonzero(a) || calendar_nonzero(b) {
        if a.years != 0 || a.months != 0 || a.weeks != 0 || b.years != 0 || b.months != 0 || b.weeks != 0 return 0, .{ .RELATIVE_REQUIRED, "calendar comparison requires relative value", -1 };
    }

    left := clock_ns(a) + wide_mul(a.days, DAY_NS);
    right := clock_ns(b) + wide_mul(b.days, DAY_NS);

    return wide_compare(left, right), .{};
}

// Uses a relative Date; e.g. `span_compare(a, b, origin)`.
span_compare :: (a: Span, b: Span, relative: Date) -> int, Time_Error {
    return compare_relative(a, b, Date_Time.{ relative, .{} });
}

// Uses a relative Zoned; e.g. `span_compare(a, b, origin)`.
span_compare :: (a: Span, b: Span, relative: Zoned) -> int, Time_Error {
    return compare_relative(a, b, relative);
}

// Converts clock units to elapsed time; calendar units require a relative value. Example: `value, error := span_to_duration(value)`.
span_to_duration :: (value: Span) -> Signed_Duration, Time_Error {
    if value.years != 0 || value.months != 0 || value.weeks != 0 return .{}, .{ .RELATIVE_REQUIRED, "calendar conversion requires relative value", -1 };
    return duration_wide(clock_ns(value) + wide_mul(cast(s64) value.days, DAY_NS));
}

// Uses a relative Date; e.g. `span_to_duration(value, origin)`.
span_to_duration :: (value: Span, relative: Date) -> Signed_Duration, Time_Error {
    origin := Date_Time.{ relative, .{} };
    end, e := add(origin, value);
    if e.kind != .NONE return .{}, e;
    return duration_wide(value_ns(end) - value_ns(origin));
}

// Uses a relative Zoned; e.g. `span_to_duration(value, origin)`.
span_to_duration :: (value: Span, relative: Zoned) -> Signed_Duration, Time_Error {
    origin := relative;
    end, e := add(origin, value);
    if e.kind != .NONE return .{}, e;
    return duration_wide(value_ns(end) - value_ns(origin));
}

// Converts clock units only; years, months, weeks and days are errors. Example: `value, error := duration_from_span(value)`.
duration_from_span :: (value: Span) -> Signed_Duration, Time_Error {
    if calendar_nonzero(value) return .{}, .{ .RELATIVE_REQUIRED, "calendar units require a relative value", -1 };
    return duration_wide(clock_ns(value));
}

// Reverses every field of a validated span. Example: `s := span_negate(value)`.
span_negate :: (value: Span) -> Span {
    return .{ -value.years, -value.months, -value.weeks, -value.days, -value.hours, -value.minutes, -value.seconds, -value.milliseconds, -value.microseconds, -value.nanoseconds };
}

// Returns a nonnegative span; span_check rejects signed minima. Example: `s := span_abs(value)`.
span_abs :: (value: Span) -> Span {
    for field: span_fields(value) {
        if field < 0 return span_negate(value);
        if field > 0 return value;
    }

    return value;
}

#scope_module

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

clock_ns :: (v: Span) -> S128 {
    return wide_mul(v.hours, 3600 * NS) + wide_mul(v.minutes, 60 * NS) + wide_mul(v.seconds, NS) + wide_mul(v.milliseconds, 1000000) + wide_mul(v.microseconds, 1000) + to_s128(v.nanoseconds);
}

unit_scale :: (u: Unit) -> s64 {
    if u == .NANOSECOND return 1;
    if u == .MICROSECOND return 1000;
    if u == .MILLISECOND return 1000000;
    if u == .SECOND return NS;
    if u == .MINUTE return 60 * NS;
    if u == .HOUR return 3600 * NS;

    return DAY_NS;
}

unit_field :: (v: Span, u: Unit) -> s64 {
    if u == .YEAR return v.years;
    if u == .MONTH return v.months;
    if u == .WEEK return v.weeks;
    if u == .DAY return v.days;
    if u == .HOUR return v.hours;
    if u == .MINUTE return v.minutes;
    if u == .SECOND return v.seconds;
    if u == .MILLISECOND return v.milliseconds;
    if u == .MICROSECOND return v.microseconds;

    return v.nanoseconds;
}

put_unit :: (v: *Span, u: Unit, n: s64) {
    if u == .YEAR v.years = cast,trunc(s32) n;
    else if u == .MONTH v.months = cast,trunc(s32) n;
    else if u == .WEEK v.weeks = cast,trunc(s32) n;
    else if u == .DAY v.days = cast,trunc(s32) n;
    else if u == .HOUR v.hours = n;
    else if u == .MINUTE v.minutes = n;
    else if u == .SECOND v.seconds = n;
    else if u == .MILLISECOND v.milliseconds = n;
    else if u == .MICROSECOND v.microseconds = n;
    else v.nanoseconds = n;
}

round_up :: (mode: Rounding, negative: bool, parity: bool, dist: S128, other: S128) -> bool {
    if mode == .TRUNC return false;
    if mode == .EXPAND return true;
    if mode == .FLOOR return negative;
    if mode == .CEIL return !negative;
    if dist != other return dist > other;
    if mode == .HALF_TRUNC return false;
    if mode == .HALF_EVEN return parity;
    if mode == .HALF_FLOOR return negative;
    if mode == .HALF_CEIL return !negative;

    return true;
}

round_wide :: (v: S128, step: S128, mode: Rounding) -> S128 {
    q, r, _ := signed_divide_with_remainder(v, step);
    if r == S128.{} return v;

    base := v - r;

    if round_up(mode, v.high < 0, (q.low & 1) != 0, wide_abs(r), step - wide_abs(r)) {
        if v.high < 0 base -= step;
        else base += step;
    }

    return base;
}

split_time :: (ns: S128, largest: Unit) -> Span, Time_Error {
    if largest > .DAY return .{}, .{ .INVALID_VALUE, "invalid exact largest unit", -1 };

    result: Span;
    remaining := ns;

    for < i: 0..cast(s64) largest {
        u := cast(Unit) i;
        q, r := wide_div(remaining, unit_scale(u));
        bound := ifx u == .DAY then 2147483647 else MAX_I64;

        if q > to_s128(bound) || q < to_s128(-bound) return .{}, range_error();

        put_unit(*result, u, narrow(q));
        remaining = to_s128(r);
    }

    return result, .{};
}

value_ns :: (v: $T) -> S128 {
    #if T == Date return wide_mul(date_days(v), DAY_NS);
    else #if T == Date_Time return wide_mul(date_days(v.date), DAY_NS) + to_s128(time_ns(v.time));
    else #if T == Timestamp return stamp_ns(v);
    else return stamp_ns(v.timestamp);
}

calendar_limit :: (u: Unit) -> s64 {
    if u == .YEAR return 20000;
    if u == .MONTH return 240000;
    if u == .WEEK return 1100000;
    return 7400000;
}

// Whether a plus `n` of `u` (on top of base, in direction sign) stays on a's side of b.
unit_fits :: (a: $T, b: T, base: Span, u: Unit, sign: s64, n: s64, policy: Disambiguation) -> bool {
    trial := base;

    put_unit(*trial, u, n * sign);

    #if T == Zoned {
        v, e := add(a, trial, disambiguation = policy);
        return e.kind == .NONE && wide_compare(value_ns(v), value_ns(b)) * sign <= 0;
    } else {
        v, e := add(a, trial);
        return e.kind == .NONE && wide_compare(value_ns(v), value_ns(b)) * sign <= 0;
    }
}

// The calendar date of v in a's calendar: v itself, or the instant seen in a's zone.
civil_date :: (a: $T, v: T) -> Date {
    #if T == Date return v;
    else #if T == Date_Time return v.date;
    else return to_civil(a.zone, v.timestamp).date;
}

// An estimate of how many `u` from a, after base, reach b, from the calendar fields alone. It
// can be off by a unit or two (clock readings, month ends, zone changes); unit_fits settles it.
unit_guess :: (a: $T, b: T, base: Span, u: Unit, sign: s64) -> s64 {
    start := civil_date(a, a);
    finish := civil_date(a, b);
    months := (cast(s64) finish.year - start.year) * 12 + finish.month - start.month;

    if u == .YEAR return max(months * sign / 12, 0);
    if u == .MONTH return max(months * sign - cast(s64) base.years * sign * 12, 0);

    // Weeks and days count from the date the larger units already reach.
    calendar := base;

    clear_clock(*calendar);
    reached, e := add(start, calendar);
    if e.kind != .NONE return 0;

    days := (date_days(finish) - date_days(reached)) * sign;

    return max(ifx u == .WEEK then days / 7 else days, 0);
}

difference_raw :: (a: $T, b: T, largest: Unit, stop: Unit, policy: Disambiguation) -> Span, Time_Error {
    result: Span;
    sign := wide_compare(value_ns(b), value_ns(a));

    if sign == 0 return result, .{};

    #if T != Timestamp {
        for < i: cast(s64) stop..cast(s64) largest {
            u := cast(Unit) i;
            if u < .DAY || (u == .WEEK && largest != .WEEK) continue;

            // Find the largest count that does not pass b. Counts that fit are monotonic, so
            // start at the calendar estimate (usually within one of the answer), widen
            // geometrically to bracket it, then bisect.
            lo: s64;
            hi := calendar_limit(u);
            guess := unit_guess(a, b, result, u, sign);
            up := true;

            if guess > hi guess = hi;

            if guess > 0 {
                if unit_fits(a, b, result, u, sign, guess, policy) lo = guess;
                else {
                    hi = guess - 1;
                    up = false;
                }
            }

            step: s64 = 1;

            while lo < hi {
                probe := hi - step + 1;

                if up {
                    probe = lo + step;
                    if probe > hi probe = hi;
                }

                if probe <= lo break;

                if unit_fits(a, b, result, u, sign, probe, policy) {
                    lo = probe;
                    if !up break;
                } else {
                    hi = probe - 1;
                    if up break;
                }

                step *= 2;
            }

            while lo < hi {
                mid := lo + (hi - lo + 1) / 2;

                if unit_fits(a, b, result, u, sign, mid, policy) lo = mid;
                else hi = mid - 1;
            }

            put_unit(*result, u, lo * sign);
        }
    }

    #if T == Date {
        return result, .{};
    } else {
        current: T;
        #if T == Timestamp current = a;
        else #if T == Zoned {
            v, e := add(a, result, disambiguation = policy);
            if e.kind != .NONE return .{}, e;
            current = v;
        } else {
            v, e := add(a, result);
            if e.kind != .NONE return .{}, e;
            current = v;
        }

        if stop >= .DAY return result, .{};

        top := ifx largest >= .DAY then Unit.HOUR else largest;
        clock, clock_error := split_time(value_ns(b) - value_ns(current), top);

        if clock_error.kind != .NONE return .{}, clock_error;

        result.hours = clock.hours;
        result.minutes = clock.minutes;
        result.seconds = clock.seconds;
        result.milliseconds = clock.milliseconds;
        result.microseconds = clock.microseconds;
        result.nanoseconds = clock.nanoseconds;

        return result, .{};
    }
}

difference :: (a: $T, b: T, largest: Unit, smallest: Unit, mode: Rounding, increment: s64, policy := Disambiguation.COMPATIBLE) -> Span, Time_Error {
    small := smallest;

    #if T == Date {
        if small == .NANOSECOND small = .DAY;
        if small < .DAY || largest < .DAY return .{}, .{ .INVALID_VALUE, "date differences require calendar units", -1 };
    }

    #if T == Timestamp {
        if largest >= .DAY return .{}, .{ .INVALID_VALUE, "instant differences require clock units", -1 };
    }

    if largest > .YEAR || small > largest || increment < 1 return .{}, .{ .INVALID_VALUE, "invalid difference units or increment", -1 };

    result, e := difference_raw(a, b, largest, small, policy);
    if e.kind != .NONE return .{}, e;

    if small < .DAY {
        ns := clock_ns(result);
        rounded := round_wide(ns, wide_mul(unit_scale(small), increment), mode);

        // Nothing was rounded away, so the exact difference is already balanced.
        if rounded == ns return result, .{};

        clear_clock(*result);
        clock, ce := split_time(rounded, ifx largest >= .DAY then Unit.HOUR else largest);
        if ce.kind != .NONE return .{}, ce;

        result.hours = clock.hours;
        result.minutes = clock.minutes;
        result.seconds = clock.seconds;
        result.milliseconds = clock.milliseconds;
        result.microseconds = clock.microseconds;
        result.nanoseconds = clock.nanoseconds;
    } else {
        count := unit_field(result, small);
        sign := wide_compare(value_ns(b), value_ns(a));

        if sign == 0 return result, .{};

        base := count / increment * increment;
        changed := base != count;

        put_unit(*result, small, base);

        #if T != Timestamp {
            lower, le := add(a, result);
            if le.kind != .NONE return .{}, le;
            if value_ns(lower) == value_ns(b) return result, .{};
            if increment > 2147483647 return .{}, range_error();

            trial := result;

            put_unit(*trial, small, base + sign * increment);
            upper, ue := add(a, trial);
            if ue.kind != .NONE return .{}, ue;

            if round_up(mode, sign < 0, (base / increment) % 2 != 0, wide_abs(value_ns(b) - value_ns(lower)), wide_abs(value_ns(upper) - value_ns(b))) {
                result = trial;
                changed = true;
            }
        }

        // Only a rounded unit can need balancing into the next larger one.
        if !changed return result, .{};
    }

    #if T == Timestamp return result, .{};
    else {
        end, ee := add(a, result);
        if ee.kind != .NONE return .{}, ee;
        return difference_raw(a, end, largest, .NANOSECOND, policy);
    }
}

balance_relative :: (v: Span, largest: Unit, origin: $T) -> Span, Time_Error {
    end, e := add(origin, v);
    if e.kind != .NONE return .{}, e;
    return difference(origin, end, largest, .NANOSECOND, .TRUNC, 1);
}

round_relative :: (v: Span, smallest: Unit, largest: Unit, mode: Rounding, increment: s64, origin: $T) -> Span, Time_Error {
    end, e := add(origin, v);
    if e.kind != .NONE return .{}, e;
    return difference(origin, end, largest, smallest, mode, increment);
}

compare_relative :: (a: Span, b: Span, origin: $T) -> int, Time_Error {
    left, e := add(origin, a);
    if e.kind != .NONE return 0, e;

    right, re := add(origin, b);
    if re.kind != .NONE return 0, re;

    return wide_compare(value_ns(left), value_ns(right)), .{};
}

total_relative :: (v: Span, unit: Unit, origin: $T) -> float64, Time_Error {
    if unit > .YEAR return 0, .{ .INVALID_VALUE, "invalid total unit", -1 };

    end, e := add(origin, v);
    if e.kind != .NONE return 0, e;
    if unit < .DAY return wide_float(value_ns(end) - value_ns(origin)) / cast(float64) unit_scale(unit), .{};

    diff, de := difference_raw(origin, end, unit, unit, .COMPATIBLE);
    if de.kind != .NONE return 0, de;

    count := unit_field(diff, unit);
    base, be := add(origin, diff);

    if be.kind != .NONE return 0, be;

    rest := value_ns(end) - value_ns(base);
    if rest == S128.{} return cast(float64) count, .{};

    sign := ifx rest.high < 0 then -1 else 1;

    put_unit(*diff, unit, count + sign);
    next, ne := add(origin, diff);
    if ne.kind != .NONE return 0, ne;

    length := wide_abs(value_ns(next) - value_ns(base));
    if length == S128.{} return 0, .{ .INVALID_VALUE, "zero-length calendar unit", -1 };

    return cast(float64) count + wide_float(rest) / wide_float(length), .{};
}

compare_sign :: (v: Span) -> s64 {
    for field: span_fields(v) {
        if field < 0 return -1;
        if field > 0 return 1;
    }

    return 0;
}
