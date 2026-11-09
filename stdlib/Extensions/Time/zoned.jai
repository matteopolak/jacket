// Offset lookup, zoned resolution across gaps and folds, and transitions.

// Resolves the offset at an exact instant, using footer rules beyond stored transitions. Example: `o := offset_at(zone, instant)`.
offset_at :: (zone: Time_Zone, instant: Timestamp) -> Offset {
    if zone.kind == .FIXED return zone.offset;
    if zone.kind != .RULES || zone.rules == null return .{};

    r := lookup_rules(zone.rules);
    if r == null return .{};

    second := instant.seconds;

    if instant.nanoseconds < 0 second -= 1;

    index := zone_type(r, second);
    if index < 0 {
        off, _ := posix_at(r.footer, second);
        return .{ cast(s32) off };
    }

    return .{ cast(s32) be_signed(r.source, r.type_at + index * 6, 4) };
}

// Returns a zone-borrowed abbreviation for formatting and conformance. Example: `text := abbreviation_at(zone, instant)`.
abbreviation_at :: (zone: Time_Zone, instant: Timestamp) -> string {
    if zone.kind == .UTC return "UTC";
    if zone.kind != .RULES || zone.rules == null return "";

    r := lookup_rules(zone.rules);
    if r == null return .{};

    second := instant.seconds;

    if instant.nanoseconds < 0 second -= 1;

    index := zone_type(r, second);
    if index < 0 {
        _, name := posix_at(r.footer, second);
        return name;
    }

    at := r.ab + r.source[r.type_at + index * 6 + 5];
    end := at;

    while end < r.ab + r.chars && r.source[end] != 0 end += 1;

    return string.{ end - at, r.source.data + at };
}

// Converts an instant in an explicit zone, rejecting civil range overflow. Example: `wall := to_civil(zone, instant)`.
to_civil :: (zone: Time_Zone, instant: Timestamp) -> Date_Time {
    seconds := instant.seconds + offset_at(zone, instant).seconds;
    ns: s64 = instant.nanoseconds;

    if ns < 0 {
        seconds -= 1;
        ns += NS;
    }

    return .{ civil_day(floor_div(seconds, 86400)), clock_from_ns(modulo(seconds, 86400) * NS + ns) };
}

// Resolves gaps/folds using the requested policy and checks UTC range. Example: `instant, error := to_timestamp(zone, wall, .REJECT)`.
to_timestamp :: (zone: Time_Zone, wall: Date_Time, disambiguation := Disambiguation.COMPATIBLE) -> Timestamp, Time_Error {
    d, e := date(wall.date.year, wall.date.month, wall.date.day);
    if e.kind != .NONE return .{}, e;

    t, te := time_of_day(wall.time.hour, wall.time.minute, wall.time.second, wall.time.nanosecond);
    if te.kind != .NONE return .{}, te;

    local := date_days(d) * 86400 + time_ns(t) / NS;
    ns := time_ns(t) % NS;
    r: *Zone_Rules;

    if zone.kind == .RULES r = lookup_rules(zone.rules);

    offsets: [258] s64;
    count: s64 = 1;

    offsets[0] = zone.offset.seconds;
    if zone.kind == .UTC offsets[0] = 0;

    if r != null {
        count = 0;

        for i: 0..r.types - 1 {
            offsets[count] = be_signed(r.source, r.type_at + i * 6, 4);
            count += 1;
        }

        if r.footer.present {
            offsets[count] = r.footer.standard;
            count += 1;

            if r.footer.dst {
                offsets[count] = r.footer.daylight;
                count += 1;
            }
        }
    }

    found := false;
    early, late: Timestamp;

    for i: 0..count - 1 {
        candidate := raw_stamp(local - offsets[i], ns);
        if offset_at(zone, candidate).seconds != offsets[i] continue;

        if !found {
            early = candidate;
            late = candidate;
            found = true;
        } else {
            if compare(candidate, early) < 0 early = candidate;
            if compare(candidate, late) > 0 late = candidate;
        }
    }

    if found {
        if compare(early, late) != 0 && disambiguation == .REJECT return .{}, .{ .FOLD, "ambiguous civil time", -1 };
        chosen := ifx disambiguation == .LATER then late else early;
        return timestamp_wide(stamp_ns(chosen));
    }

    if r != null {
        cursor := transitions(zone, raw_stamp(local - 187200, 0));

        // Only transitions in a bounded four-day window can affect this local reading.
        for 1..r.count + 8 {
            change, has, error := next(*cursor);
            if error.kind != .NONE return .{}, error;
            if !has || change.timestamp.seconds > local + 187200 break;

            before := change.timestamp.seconds + change.before.seconds;
            after := change.timestamp.seconds + change.after.seconds;

            if before <= local && local < after {
                if disambiguation == .REJECT return .{}, .{ .GAP, "nonexistent civil time", -1 };
                off := ifx disambiguation == .EARLIER then change.after else change.before;
                return timestamp_wide(wide_mul(local - off.seconds, NS) + to_s128(ns));
            }
        }
    }

    return .{}, range_error();
}

// Starts a borrowed cursor strictly after instant (strictly before when reverse). Example: `cursor := transitions(zone, instant)`.
transitions :: (zone: Time_Zone, instant: Timestamp, reverse := false) -> Time_Transitions {
    return .{ zone, instant, reverse, false };
}

// Advances a transition cursor; false with no error means the range is exhausted. Example: `change, found, error := next(*cursor)`.
next :: (cursor: *Time_Transitions) -> Time_Transition, bool, Time_Error {
    if cursor.exhausted || cursor.zone.kind != .RULES || cursor.zone.rules == null return .{}, false, .{};

    r := lookup_rules(cursor.zone.rules);
    if r == null return .{}, false, .{};

    found := false;
    chosen: s64;
    lo: s64;
    hi := r.count;

    while lo < hi {
        mid := lo + (hi - lo) / 2;
        seconds := be_signed(r.source, r.ta + mid * r.width, r.width);
        c := compare(Timestamp.{ seconds, 0 }, cursor.cursor);

        if c < 0 || (!cursor.reverse && c == 0) lo = mid + 1;
        else hi = mid;
    }

    index := ifx cursor.reverse then lo - 1 else lo;
    if index >= 0 && index < r.count {
        chosen = be_signed(r.source, r.ta + index * r.width, r.width);
        found = true;
    }

    if r.footer.present && r.footer.dst {
        y: s64 = to_civil(time_zone_utc(), cursor.cursor).date.year;
        last := MIN_I64;

        if r.count > 0 last = be_signed(r.source, r.ta + (r.count - 1) * r.width, r.width);

        for year: y - 1..y + 1 {
            for event: s64.[rule_in_year(r.footer.start, year, r.footer.standard), rule_in_year(r.footer.end, year, r.footer.daylight)] {
                c := compare(Timestamp.{ event, 0 }, cursor.cursor);
                if event > last && ((cursor.reverse && c < 0) || (!cursor.reverse && c > 0)) && (!found || (cursor.reverse && event > chosen) || (!cursor.reverse && event < chosen)) {
                    chosen = event;
                    found = true;
                }
            }
        }
    }

    if !found {
        cursor.exhausted = true;
        return .{}, false, .{};
    }

    instant, e := timestamp_from_seconds(chosen);
    if e.kind != .NONE {
        cursor.exhausted = true;
        return .{}, false, .{};
    }

    before := Timestamp.{ chosen - 1, 0 };

    cursor.cursor = instant;

    return .{ instant, offset_at(cursor.zone, before), offset_at(cursor.zone, instant), abbreviation_at(cursor.zone, before), abbreviation_at(cursor.zone, instant) }, true, .{};
}

// Resolves an instant and borrows its zone; the zone owner must outlive the result. Example: `z := zoned(instant, zone)`.
zoned :: (instant: Timestamp, zone: Time_Zone) -> Zoned {
    return .{ instant, zone, offset_at(zone, instant) };
}

// Resolves civil time and borrows the explicit zone. Example: `z, error := to_zoned(wall, zone, .LATER)`.
to_zoned :: (wall: Date_Time, zone: Time_Zone, disambiguation := Disambiguation.COMPATIBLE) -> Zoned, Time_Error {
    t, e := to_timestamp(zone, wall, disambiguation);
    if e.kind != .NONE return .{}, e;
    return zoned(t, zone), .{};
}

// Reads the existing Basic consensus clock in an explicit borrowed zone. Example: `z, error := zoned_now(zone)`.
zoned_now :: (zone: Time_Zone) -> Zoned, Time_Error {
    t, e := timestamp_now();
    if e.kind != .NONE return .{}, e;
    return zoned(t, zone), .{};
}

// Keeps the instant while borrowing a different zone and resolving its offset. Example: `z := with_zone(value, time_zone_utc())`.
with_zone :: (value: Zoned, zone: Time_Zone) -> Zoned {
    return .{ value.timestamp, zone, offset_at(zone, value.timestamp) };
}

// Adds calendar units in civil time, resolves, then adds clock units on the timeline. Example: `z, error := add(value, Span.{days = 1})`.
add :: (value: Zoned, amount: Span, overflow := Overflow.CONSTRAIN, disambiguation := Disambiguation.COMPATIBLE) -> Zoned, Time_Error {
    instant := value.timestamp;

    if calendar_nonzero(amount) {
        calendar := amount;

        clear_clock(*calendar);
        wall, e := add(to_civil(value.zone, instant), calendar, overflow);
        if e.kind != .NONE return .{}, e;

        instant, e = to_timestamp(value.zone, wall, disambiguation);
        if e.kind != .NONE return .{}, e;
    }

    result, e := timestamp_wide(stamp_ns(instant) + clock_ns(amount));
    if e.kind != .NONE return .{}, e;

    return zoned(result, value.zone), .{};
}

// Finds the first valid instant of the local date, including days starting after midnight. Example: `start, error := start_of_day(value)`.
start_of_day :: (value: Zoned) -> Zoned, Time_Error {
    wall := to_civil(value.zone, value.timestamp);
    wall.time = .{};
    t, e := to_timestamp(value.zone, wall);
    if e.kind != .NONE return .{}, e;

    if compare(to_civil(value.zone, t), wall) > 0 {
        cursor := transitions(value.zone, raw_stamp(t.seconds + 1, 0), true);
        change, has, _ := next(*cursor);
        if has && compare(to_civil(value.zone, change.timestamp).date, wall.date) == 0 t = change.timestamp;
    }

    return zoned(t, value.zone), .{};
}

// Returns exact elapsed hours between consecutive local day starts. Example: `hours, error := hours_in_day(value)`.
hours_in_day :: (value: Zoned) -> float64, Time_Error {
    start, e := start_of_day(value);
    if e.kind != .NONE return 0, e;

    wall := to_civil(value.zone, start.timestamp);
    d, de := add(wall.date, Span.{ days = 1 });

    if de.kind != .NONE return 0, de;

    following, fe := to_zoned(.{ d, .{} }, value.zone);
    if fe.kind != .NONE return 0, fe;

    end, ee := start_of_day(following);
    if ee.kind != .NONE return 0, ee;

    delta := duration_between(start.timestamp, end.timestamp);

    return (cast(float64) delta.seconds + cast(float64) delta.nanoseconds / NS) / 3600, .{};
}

#scope_module

zone_type :: (r: *Zone_Rules, second: s64) -> s64 {
    if r.footer.present && (r.count == 0 || second > be_signed(r.source, r.ta + (r.count - 1) * r.width, r.width)) return -1;

    lo: s64;
    hi := r.count;

    while lo < hi {
        mid := lo + (hi - lo) / 2;
        if be_signed(r.source, r.ta + mid * r.width, r.width) <= second lo = mid + 1;
        else hi = mid;
    }

    if lo == 0 return 0;

    return r.source[r.ia + lo - 1];
}

rule_in_year :: (rule: Zone_Rule, year: s64, offset: s64) -> s64 {
    ordinal := rule.a;

    if rule.kind == 2 {
        first := Date.{ cast(s16) year, cast(s8) rule.a, 1 };
        day := 1 + modulo(rule.c - (cast(s64) weekday(first) % 7), 7) + (rule.b - 1) * 7;
        if day > month_length(year, rule.a) day -= 7;
        ordinal = date_days(.{ cast(s16) year, cast(s8) rule.a, cast(s8) day }) - date_days(.{ cast(s16) year, 1, 1 });
    } else if rule.kind == 0 {
        ordinal -= 1;
        if is_leap_year(year) && rule.a >= 60 ordinal += 1;
    }

    return (date_days(.{ cast(s16) year, 1, 1 }) + ordinal) * 86400 + rule.seconds - offset;
}

posix_at :: (p: Posix_Rules, seconds: s64) -> s64, string {
    if !p.dst return p.standard, p.standard_name;

    year: s64 = civil_day(floor_div(seconds, 86400)).year;
    // Extended transition times can cross the year boundary: choose the last event.
    last: s64 = MIN_I64;
    dst := false;

    for y: year - 1..year + 1 {
        start := rule_in_year(p.start, y, p.standard);
        end := rule_in_year(p.end, y, p.daylight);

        if start <= seconds && start > last {
            last = start;
            dst = true;
        }

        if end <= seconds && end > last {
            last = end;
            dst = false;
        }
    }

    if dst return p.daylight, p.daylight_name;

    return p.standard, p.standard_name;
}
