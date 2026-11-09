// Pattern text: English strftime-style formatting and parsing.

// Reads the documented English directives; missing or conflicting required fields fail. Example: `wall, error := parse_format(Date_Time, "%Y-%m-%d %H:%M", "2024-03-10 02:30")`.
parse_format :: ($T: Type, pattern: string, text: string, options: Time_Options = .{}) -> T, Time_Error {
    if !valid_options(options) || text.count > options.max_text || pattern.count > options.max_text return .{}, .{ .LIMIT, "pattern text limit", 0 };

    r := Text_Reader.{ text = text };
    p := Text_Reader.{ text = pattern };
    v: Pattern_Parse;

    while p.at < pattern.count && r.error.kind == .NONE {
        c := peek(*p);

        p.at += 1;

        if c != 37 {
            expect_byte(*r, c);
            continue;
        }

        d, pad, width := pattern_directive(*p);

        if p.error.kind != .NONE {
            text_fail(*r, .SYNTAX, "invalid pattern");
            break;
        }

        if d == 37 {
            expect_byte(*r, 37);
            continue;
        }

        if d == 122 || d == 99 {
            if d == 99 v.offset = read_offset(*r);
            else {
                sign := peek(*r);
                if sign != 43 && sign != 45 text_fail(*r);
                else r.at += 1;

                h := read_number(*r, 2);
                m := read_number(*r, 2);
                sec: s64;

                if peek(*r) >= 48 && peek(*r) <= 57 sec = read_number(*r, 2);

                off, e := offset((h * 3600 + m * 60 + sec) * (ifx sign == 45 then -1 else 1));

                if m > 59 || sec > 59 || e.kind != .NONE text_fail(*r, .INVALID_VALUE, "invalid numeric offset");

                v.offset = off;
            }

            v.has_offset = true;
            continue;
        }

        if d == 81 || d == 90 {
            read_pattern_zone(*r, *v, d);
            continue;
        }

        if d == 97 || d == 65 || d == 98 || d == 66 || d == 112 {
            read_pattern_word(*r, *v, d);
            continue;
        }

        if !contains("Ymd eHIMSfjGVusF", d) {
            text_fail(*r, .SYNTAX, "unknown pattern directive");
            break;
        }

        if d == 70 {
            ns: s64;
            count: s64;

            if peek(*r) == 46 {
                r.at += 1;
                while peek(*r) >= 48 && peek(*r) <= 57 && count < 9 {
                    ns = ns * 10 + peek(*r) - 48;
                    r.at += 1;
                    count += 1;
                }

                if count == 0 text_fail(*r);
                for count..8 ns *= 10;
            }

            set_pattern_number(*r, *v, 102, ns);
            continue;
        }

        sign: s64 = 1;

        if d == 89 || d == 71 || d == 115 {
            if peek(*r) == 45 || peek(*r) == 43 {
                if peek(*r) == 45 sign = -1;
                r.at += 1;
                if d != 115 width = 6;
            }
        }

        if pad == 32 {
            for 1..width - 1 {
                if peek(*r) != 32 break;
                r.at += 1;
                width -= 1;
            }
        }

        n: s64;
        if width == 0 || d == 115 n = read_variable_checked(*r);
        else n = read_number(*r, width);

        set_pattern_number(*r, *v, d, n * sign);
    }

    if r.error.kind == .NONE && r.at != text.count text_fail(*r);
    if r.error.kind != .NONE return .{}, r.error;

    return finish_pattern(T, *v, *r, options);
}

#scope_module

WEEKDAYS :: string.["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

MONTHS :: string.["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

Pattern_Fields :: struct {
    wall: Date_Time;
    has_date: bool;
    has_time: bool;
    has_offset: bool;
    has_stamp: bool;
    offset: Offset;
    stamp: Timestamp;
    name: string;
    abbreviation: string;
}

pattern_fields :: (v: $T) -> Pattern_Fields {
    f: Pattern_Fields;

    #if T == Date {
        f.wall.date = v;
        f.has_date = true;
    } else #if T == Time_Of_Day {
        f.wall.time = v;
        f.has_time = true;
    } else #if T == Date_Time {
        f.wall = v;
        f.has_date = true;
        f.has_time = true;
    } else #if T == Timestamp || T == Zoned {
        zone: Time_Zone;

        #if T == Timestamp f.stamp = v;
        else {
            f.stamp = v.timestamp;
            zone = v.zone;
        }

        f.wall = to_civil(zone, f.stamp);
        f.offset = offset_at(zone, f.stamp);
        f.abbreviation = abbreviation_at(zone, f.stamp);
        f.has_date = true;
        f.has_time = true;
        f.has_offset = true;
        f.has_stamp = true;
        if zone.kind == .UTC f.name = "UTC";
        else if zone.kind == .RULES {
            r := lookup_rules(zone.rules);
            if r != null f.name = r.name;
        }
    } else #if T == Offset {
        f.offset = v;
        f.has_offset = true;
    } else #if T == Time_Zone {
        f.offset = v.offset;
        f.has_offset = v.kind != .RULES;

        if v.kind == .UTC {
            f.name = "UTC";
            f.abbreviation = "UTC";
        } else if v.kind == .RULES {
            r := lookup_rules(v.rules);
            if r != null f.name = r.name;
        }
    } else #if T == Signed_Duration {
        f.stamp = .{ v.seconds, v.nanoseconds };
        f.has_stamp = true;
    }

    return f;
}

floor_seconds :: (v: Timestamp) -> s64 {
    return v.seconds - (ifx v.nanoseconds < 0 then 1 else 0);
}

pattern_directive :: (p: *Text_Reader) -> u8, u8, s64 {
    pad: u8 = 48;
    width: s64 = 2;
    c := peek(p);

    if c == 45 {
        width = 0;
        p.at += 1;
    } else if c == 95 {
        pad = 32;
        p.at += 1;
    }

    c = peek(p);
    p.at += 1;

    if c == 46 {
        expect_byte(p, 102);
        return 70, pad, 0;
    }

    if c == 58 {
        expect_byte(p, 122);
        return 99, pad, 0;
    }

    if width != 0 {
        if c == 89 || c == 71 width = 4;
        else if c == 106 width = 3;
        else if c == 102 width = 9;
        else if c == 117 width = 1;
    }

    if c == 101 pad = 32;

    return c, pad, width;
}

Pattern_Parse :: struct {
    values: [16] s64;
    seen: [16] bool;
    ampm: s64 = -1;
    weekday: s64;
    seen_weekday: bool;
    has_offset: bool;
    offset: Offset;
    name: string;
    abbreviation: string;
}

pattern_index :: (d: u8) -> s64 {
    for c, i: "YmdHMSfjGVusI" if c == d return i;
    if d == 101 return 2;
    return -1;
}

set_pattern_number :: (r: *Text_Reader, v: *Pattern_Parse, d: u8, n: s64) {
    i := pattern_index(d);
    if i < 0 {
        text_fail(r);
        return;
    }

    if v.seen[i] && v.values[i] != n text_fail(r, .INVALID_VALUE, "conflicting pattern fields");

    v.values[i] = n;
    v.seen[i] = true;
}

read_variable_checked :: (r: *Text_Reader) -> s64 {
    n: s64;
    begin := r.at;

    while peek(r) >= 48 && peek(r) <= 57 {
        d: s64 = peek(r) - 48;
        if n > (MAX_I64 - d) / 10 {
            text_fail(r, .OUT_OF_RANGE, "pattern integer overflow");
            break;
        }

        n = n * 10 + d;
        r.at += 1;
    }

    if begin == r.at text_fail(r);

    return n;
}

consume_word :: (r: *Text_Reader, word: string) -> bool {
    if r.text.count - r.at < word.count return false;
    if !equal_nocase(string.{ word.count, r.text.data + r.at }, word) return false;
    r.at += word.count;
    return true;
}

finish_pattern :: ($T: Type, v: *Pattern_Parse, r: *Text_Reader, options: Time_Options) -> T, Time_Error {
    zone: Time_Zone;
    keep := false;

    defer if !keep time_zone_free(*zone);

    if v.name.count != 0 {
        if v.name != "UTC" {
            z, e := time_zone_get(v.name, options);
            if e.kind != .NONE {
                e.offset = r.at;
                return .{}, e;
            }

            zone = z;
        }
    } else if v.has_offset zone = time_zone_fixed(v.offset);
    else if v.abbreviation == "UTC" || v.abbreviation == "GMT" {
        v.has_offset = true;
    } else if v.abbreviation.count != 0 return .{}, .{ .INVALID_VALUE, "abbreviation requires an explicit zone", r.at };

    #if T == Time_Zone {
        if v.name.count == 0 && !v.has_offset return .{}, .{ .INVALID_VALUE, "zone field required", r.at };
        keep = true;
        return zone, .{};
    }

    #if T == Offset {
        if !v.has_offset return .{}, .{ .INVALID_VALUE, "offset field required", r.at };
        return v.offset, .{};
    }

    d: Date;
    t: Time_Of_Day;
    has_date := false;

    if v.seen[0] && v.seen[1] && v.seen[2] {
        value, e := date(v.values[0], v.values[1], v.values[2]);
        if e.kind != .NONE {
            e.offset = r.at;
            return .{}, e;
        }

        d = value;
        has_date = true;
    }

    if v.seen[7] {
        if !v.seen[0] || v.values[7] < 1 || v.values[7] > 366 return .{}, .{ .INVALID_VALUE, "ordinal needs a valid year and day", r.at };

        first, e := date(v.values[0], 1, 1);
        if e.kind != .NONE {
            e.offset = r.at;
            return .{}, e;
        }

        value, ve := checked_day(date_days(first) + v.values[7] - 1);
        if ve.kind != .NONE || value.year != first.year || (has_date && compare(d, value) != 0) return .{}, .{ .INVALID_VALUE, "conflicting ordinal date", r.at };

        d = value;
        has_date = true;
    }

    if v.seen[8] || v.seen[9] {
        if !v.seen[8] || !v.seen[9] || !v.seen[10] return .{}, .{ .INVALID_VALUE, "incomplete ISO week date", r.at };

        if v.values[10] < 1 || v.values[10] > 7 return .{}, .{ .INVALID_VALUE, "invalid ISO weekday", r.at };

        value, e := date_from_iso_week(v.values[8], v.values[9], cast(Weekday) v.values[10]);
        if e.kind != .NONE || (has_date && compare(d, value) != 0) return .{}, .{ .INVALID_VALUE, "conflicting ISO week date", r.at };

        d = value;
        has_date = true;
    }

    if has_date && ((v.seen[0] && v.values[0] != d.year) || (v.seen[1] && v.values[1] != d.month) || (v.seen[2] && v.values[2] != d.day) || (v.seen[10] && v.values[10] != cast(s64) weekday(d))) return .{}, .{ .INVALID_VALUE, "conflicting civil fields", r.at };
    if v.seen_weekday && has_date && cast(s64) weekday(d) != v.weekday return .{}, .{ .INVALID_VALUE, "weekday conflicts with date", r.at };

    hour := v.values[3];

    if v.seen[12] {
        if v.values[12] < 1 || v.values[12] > 12 || v.ampm < 0 return .{}, .{ .INVALID_VALUE, "12-hour clock requires AM or PM", r.at };
        h := (v.values[12] % 12) + v.ampm * 12;
        if v.seen[3] && h != hour return .{}, .{ .INVALID_VALUE, "conflicting hours", r.at };
        hour = h;
    }

    clock, e := time_of_day(hour, v.values[4], v.values[5], v.values[6]);
    if e.kind != .NONE {
        e.offset = r.at;
        return .{}, e;
    }

    t = clock;

    #if T == Date {
        if !has_date return .{}, .{ .INVALID_VALUE, "complete date required", r.at };
        return d, .{};
    }

    #if T == Time_Of_Day {
        if !v.seen[3] && !v.seen[12] return .{}, .{ .INVALID_VALUE, "hour required", r.at };
        return t, .{};
    }

    #if T == Date_Time {
        if !has_date || (!v.seen[3] && !v.seen[12]) return .{}, .{ .INVALID_VALUE, "complete date and clock required", r.at };
        return .{ d, t }, .{};
    }

    #if T == Signed_Duration {
        if !v.seen[11] return .{}, .{ .INVALID_VALUE, "epoch seconds required", r.at };
        return signed_duration(v.values[11], v.values[6]);
    }

    #if T == Timestamp || T == Zoned {
        instant: Timestamp;

        if v.seen[11] {
            ts, te := timestamp_wide(wide_mul(v.values[11], NS) + to_s128(v.values[6]));
            if te.kind != .NONE {
                te.offset = r.at;
                return .{}, te;
            }

            instant = ts;
        } else {
            if !has_date || (v.name.count == 0 && !v.has_offset) return .{}, .{ .INVALID_VALUE, "complete date and explicit zone required", r.at };

            ts, te := to_timestamp(ifx v.has_offset then time_zone_fixed(v.offset) else zone, .{ d, t });
            if te.kind != .NONE {
                te.offset = r.at;
                return .{}, te;
            }

            instant = ts;
        }

        if v.has_offset && offset_at(zone, instant).seconds != v.offset.seconds return .{}, .{ .OFFSET_CONFLICT, "conflicting offset", r.at };
        if v.abbreviation.count != 0 && abbreviation_at(zone, instant) != v.abbreviation return .{}, .{ .INVALID_VALUE, "conflicting abbreviation", r.at };
        if v.seen[11] && has_date && compare(to_civil(zone, instant).date, d) != 0 return .{}, .{ .INVALID_VALUE, "conflicting epoch date", r.at };

        if v.seen[11] {
            wall := to_civil(zone, instant);
            if (v.seen[3] || v.seen[12]) && wall.time.hour != t.hour || v.seen[4] && wall.time.minute != t.minute || v.seen[5] && wall.time.second != t.second return .{}, .{ .INVALID_VALUE, "conflicting epoch clock", r.at };
        }

        #if T == Timestamp return instant, .{};
        else {
            keep = true;
            return zoned(instant, zone), .{};
        }
    }

    return .{}, .{ .UNSUPPORTED, "unsupported pattern type", r.at };
}

read_pattern_word :: (r: *Text_Reader, v: *Pattern_Parse, d: u8) {
    which: s64 = -1;

    if d == 112 {
        for word, i: string.["AM", "PM"] if consume_word(r, word) {
            which = i;
            break;
        }

        if which >= 0 v.ampm = which;
    } else if d == 97 || d == 65 {
        for word, i: WEEKDAYS {
            part := word;
            if d == 97 part = slice(word, 0, 3);

            if consume_word(r, part) {
                which = i;
                break;
            }
        }

        if which >= 0 {
            v.weekday = which + 1;
            v.seen_weekday = true;
        }
    } else {
        for word, i: MONTHS {
            part := word;
            if d == 98 part = slice(word, 0, 3);

            if consume_word(r, part) {
                which = i;
                break;
            }
        }

        if which >= 0 set_pattern_number(r, v, 109, which + 1);
    }

    if which < 0 text_fail(r);

}

read_pattern_zone :: (r: *Text_Reader, v: *Pattern_Parse, d: u8) {
    begin := r.at;

    while r.at < r.text.count {
        c2 := peek(r);
        if (c2 >= 65 && c2 <= 90) || (c2 >= 97 && c2 <= 122) || (c2 >= 48 && c2 <= 57) || c2 == 47 || c2 == 95 || c2 == 45 || c2 == 43 || c2 == 58 r.at += 1;
        else break;
    }

    word := string.{ r.at - begin, r.text.data + begin };
    if word.count == 0 text_fail(r);
    if d == 81 v.name = word;
    else v.abbreviation = word;

}
