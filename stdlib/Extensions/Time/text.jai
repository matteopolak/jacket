// ISO text parsing and formatting.

// Reads the type's ISO text; Zoned rejects conflicting offsets and accepts bare zone annotations. Example: `value, error := parse(Timestamp, "2024-03-10T07:30:00Z")`.
parse :: ($T: Type, text: string, options: Time_Options = .{}) -> T, Time_Error {
    if !valid_options(options) || text.count > options.max_text return .{}, .{ .LIMIT, "text limit", 0 };

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
    } else #if T == Time_Zone {
        v := read_zone(*r, options);

        if r.error.kind == .NONE && r.at != text.count {
            text_fail(*r);
            time_zone_free(*v);
        }

        return v, r.error;
    } else #if T == Zoned {
        wall := read_wall(*r);
        has_offset := peek(*r) != 91;
        o: Offset;

        if has_offset o = read_offset(*r);

        v := read_zone(*r, options);

        if r.error.kind == .NONE && r.at != text.count text_fail(*r);

        if r.error.kind != .NONE {
            time_zone_free(*v);
            return .{}, r.error;
        }

        instant: Timestamp;
        error: Time_Error;

        if has_offset {
            instant, error = to_timestamp(time_zone_fixed(o), wall);
            if error.kind == .NONE && offset_at(v, instant).seconds != o.seconds error = .{ .OFFSET_CONFLICT, "offset conflicts with zone", r.at };
        } else instant, error = to_timestamp(v, wall);

        if error.kind != .NONE {
            time_zone_free(*v);
            error.offset = r.at;
            return .{}, error;
        }

        return zoned(instant, v), .{};
    } else #if T == Span {
        v := read_span(*r);
        return v, r.error;
    } else #if T == Signed_Duration {
        v := read_duration(*r);
        return v, r.error;
    } else {
        return .{}, .{ .UNSUPPORTED, "unsupported time type", 0 };
    }
}

// Uses [String_Builder](https://github.com/matteopolak/jacket/blob/main/docs/specs/stdlib/modules/basic.md).
// Appends canonical ISO text, trims fractional zeros and uses UTC Z for timestamps. Example: `error := format(*builder, instant)`.
format :: (builder: *Builder, value: $T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned, Span]) -> Time_Error {
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
    } else #if T == Time_Zone {
        append(builder, "[");

        if value.kind == .UTC append(builder, "UTC");
        else if value.kind == .FIXED {
            e := format(builder, value.offset);
            if e.kind != .NONE return e;
        } else {
            r := lookup_rules(value.rules);
            if r == null || r.name.count == 0 return .{ .UNSUPPORTED, "unnamed zone has no ISO identifier", -1 };
            append(builder, r.name);
        }

        append(builder, "]");
    } else #if T == Zoned {
        e := format(builder, to_civil(value.zone, value.timestamp));
        if e.kind != .NONE return e;

        e = format(builder, offset_at(value.zone, value.timestamp));
        if e.kind != .NONE return e;

        return format(builder, value.zone);
    } else #if T == Span {
        v := span_abs(value);

        if compare_sign(value) < 0 append(builder, "-");

        append(builder, "P");

        if v.years != 0 {
            digits(builder, v.years);
            append(builder, "Y");
        }

        if v.months != 0 {
            digits(builder, v.months);
            append(builder, "M");
        }

        if v.weeks != 0 {
            digits(builder, v.weeks);
            append(builder, "W");
        }

        if v.days != 0 {
            digits(builder, v.days);
            append(builder, "D");
        }

        total := wide_mul(v.seconds, NS) + wide_mul(v.milliseconds, 1000000) + wide_mul(v.microseconds, 1000) + to_s128(v.nanoseconds);

        if clock_nonzero(v) || !calendar_nonzero(v) {
            append(builder, "T");

            if v.hours != 0 {
                digits(builder, v.hours);
                append(builder, "H");
            }

            if v.minutes != 0 {
                digits(builder, v.minutes);
                append(builder, "M");
            }

            if total != S128.{} || (v.hours == 0 && v.minutes == 0) {
                q, r := wide_div(total, NS);
                if q > to_s128(MAX_I64) return range_error();

                digits(builder, narrow(q));
                fraction(builder, r);
                append(builder, "S");
            }
        }
    } else #if T == Signed_Duration {
        total := wide_mul(value.seconds, NS) + to_s128(value.nanoseconds);
        if total.high < 0 append(builder, "-");

        append(builder, "PT");
        q, r := wide_div(wide_abs(total), NS);

        digits(builder, q.low);
        fraction(builder, r);
        append(builder, "S");
    } else {
        return .{ .UNSUPPORTED, "unsupported time type", -1 };
    }

    if builder.failed return .{ .LIMIT, "builder allocation failed", -1 };

    return .{};
}

// Appends English strftime text with the documented directive set and padding flags. Example: `error := format(*builder, wall, "%Y-%m-%d %H:%M:%S")`.
format :: (builder: *Builder, value: $T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned], pattern: string, options: Time_Options = .{}) -> Time_Error {
    if !valid_options(options) || pattern.count > options.max_text return .{ .LIMIT, "pattern limit", 0 };

    f := pattern_fields(value);
    p := Text_Reader.{ text = pattern };

    while p.at < pattern.count {
        c := peek(*p);

        p.at += 1;

        if c != 37 {
            append(builder, c);
            continue;
        }

        d, pad, width := pattern_directive(*p);

        if p.error.kind != .NONE return p.error;

        if d == 37 {
            append(builder, "%");
            continue;
        }

        if (contains("Ymd ejabBVGuA", d) && !f.has_date) || (contains("HIMSpfF", d) && !f.has_time) return .{ .INVALID_VALUE, "directive requires missing civil fields", p.at - 1 };

        n: s64;
        word: string;

        if d == 89 n = f.wall.date.year;
        else if d == 109 n = f.wall.date.month;
        else if d == 100 || d == 101 n = f.wall.date.day;
        else if d == 72 n = f.wall.time.hour;
        else if d == 73 {
            n = f.wall.time.hour % 12;
            if n == 0 n = 12;
        } else if d == 77 n = f.wall.time.minute;
        else if d == 83 n = f.wall.time.second;
        else if d == 102 n = f.wall.time.nanosecond;
        else if d == 70 {
            fraction(builder, f.wall.time.nanosecond);
            continue;
        } else if d == 112 word = ifx f.wall.time.hour < 12 then "AM" else "PM";
        else if d == 106 n = day_of_year(f.wall.date);
        else if d == 97 || d == 65 {
            index := cast(s64) weekday(f.wall.date) - 1;
            word = WEEKDAYS[index];
            if d == 97 word = slice(word, 0, 3);
        } else if d == 98 || d == 66 {
            index := cast(s64) f.wall.date.month - 1;
            if index < 0 || index >= 12 return .{ .INVALID_VALUE, "invalid month", -1 };
            word = MONTHS[index];
            if d == 98 word = slice(word, 0, 3);
        } else if d == 71 || d == 86 {
            year, week := iso_week(f.wall.date);
            n = ifx d == 71 then year else week;
        } else if d == 117 n = cast(s64) weekday(f.wall.date);
        else if d == 115 {
            if !f.has_stamp return .{ .INVALID_VALUE, "epoch directive needs an instant", p.at - 1 };
            n = floor_seconds(f.stamp);
        } else if d == 122 || d == 99 {
            if !f.has_offset return .{ .INVALID_VALUE, "offset directive needs a zone", p.at - 1 };

            if d == 99 {
                e := format(builder, f.offset);
                if e.kind != .NONE return e;
            } else {
                off: s64 = f.offset.seconds;

                append(builder, ifx off < 0 then "-" else "+");
                if off < 0 off = -off;

                digits(builder, off / 3600, 2);
                digits(builder, off / 60 % 60, 2);
                if off % 60 != 0 digits(builder, off % 60, 2);
            }

            continue;
        } else if d == 90 word = f.abbreviation;
        else if d == 81 word = f.name;
        else return .{ .SYNTAX, "unknown pattern directive", p.at - 1 };

        if d == 90 || d == 81 {
            if word.count == 0 return .{ .INVALID_VALUE, "zone identifier is unavailable", p.at - 1 };
        }

        if word.count != 0 append(builder, word);
        else digits(builder, n, width, pad);
    }

    if builder.failed return .{ .LIMIT, "builder allocation failed", -1 };

    return .{};
}

#scope_module

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

digits :: (b: *String_Builder, n: $N, width: s64 = 1, pad: u8 = 48) #no_aoc {
    storage: [24] u8;
    at := storage.count;
    v := cast,no_check(u64) n;

    if n < 0 {
        append(b, "-");
        v = 0 - v;
    }

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

read_zone :: (r: *Text_Reader, options: Time_Options) -> Time_Zone {
    if peek(r) != 91 {
        text_fail(r);
        return .{};
    }

    r.at += 1;
    if peek(r) == 33 r.at += 1;

    begin := r.at;

    while r.at < r.text.count && peek(r) != 93 r.at += 1;

    text := string.{ r.at - begin, r.text.data + begin };

    expect_byte(r, 93);
    if r.error.kind != .NONE return .{};
    if text == "UTC" return time_zone_utc();

    if text.count > 0 && (text[0] == 43 || text[0] == 45) {
        v, e := parse(Offset, text, options);
        if e.kind != .NONE {
            text_fail(r, e.kind, e.message, begin + e.offset);
            return .{};
        }

        return time_zone_fixed(v);
    }

    v, e := time_zone_get(text, options);
    if e.kind != .NONE text_fail(r, e.kind, e.message, begin);

    return v;
}

read_span :: (r: *Text_Reader) -> Span {
    v: Span;
    sign: s64 = 1;

    if peek(r) == 45 {
        sign = -1;
        r.at += 1;
    }

    expect_byte(r, 80);
    clock := false;
    any := false;
    time_any := false;
    last: s64 = 11;

    while r.at < r.text.count && r.error.kind == .NONE {
        if peek(r) == 84 {
            if clock {
                text_fail(r);
                break;
            }

            clock = true;
            r.at += 1;
            last = 6;
            continue;
        }

        begin := r.at;
        n: s64;

        while peek(r) >= 48 && peek(r) <= 57 {
            digit: s64 = peek(r) - 48;
            if n > (MAX_I64 - digit) / 10 {
                text_fail(r, .OUT_OF_RANGE, "span integer overflow");
                break;
            }

            n = n * 10 + digit;
            r.at += 1;
        }

        if r.at == begin {
            text_fail(r);
            break;
        }

        ns: s64;
        fractional := false;

        if peek(r) == 46 {
            fractional = true;
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

        c := peek(r);

        if r.at < r.text.count r.at += 1;

        u := Unit.NANOSECOND;

        if !clock {
            if c == 89 u = .YEAR;
            else if c == 77 u = .MONTH;
            else if c == 87 u = .WEEK;
            else if c == 68 u = .DAY;
            else text_fail(r);
        } else {
            if c == 72 u = .HOUR;
            else if c == 77 u = .MINUTE;
            else if c == 83 u = .SECOND;
            else text_fail(r);

            time_any = true;
        }

        if cast(s64) u >= last || (fractional && u != .SECOND) text_fail(r);
        if u >= .DAY && n > 2147483647 text_fail(r, .OUT_OF_RANGE, "span calendar overflow");

        put_unit(*v, u, n * sign);

        if fractional {
            v.milliseconds = ns / 1000000 * sign;
            v.microseconds = (ns / 1000 % 1000) * sign;
            v.nanoseconds = (ns % 1000) * sign;
        }

        last = cast(s64) u;
        any = true;
    }

    if !any || (clock && !time_any) text_fail(r);

    return v;
}

read_duration :: (r: *Text_Reader) -> Signed_Duration {
    negative := peek(r) == 45;
    if negative r.at += 1;

    expect_byte(r, 80);
    expect_byte(r, 84);
    begin := r.at;
    n: u64;
    limit: u64 = 9223372036854775807;

    if negative limit += 1;

    while peek(r) >= 48 && peek(r) <= 57 {
        digit: u64 = peek(r) - 48;
        if n > (limit - digit) / 10 {
            text_fail(r, .OUT_OF_RANGE, "duration integer overflow");
            return .{};
        }

        n = n * 10 + digit;
        r.at += 1;
    }

    if begin == r.at text_fail(r);

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

    expect_byte(r, 83);
    if r.at != r.text.count text_fail(r);

    total := S128.{ n, 0 };

    if negative {
        total = S128.{} - total;
        ns = -ns;
    }

    return .{ narrow(total), cast(s32) ns };
}
