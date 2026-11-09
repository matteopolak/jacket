// Offset lookup, zoned resolution across gaps and folds, and transitions.

// Resolves the offset at an exact instant, using footer rules beyond stored transitions.
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// winter, _ := timestamp_from_seconds(1704067200);
// assert(offset_at(zone, winter).seconds == -18000);
// ```
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

// Returns a zone-borrowed abbreviation for formatting and conformance.
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// summer, _ := timestamp_from_seconds(1720000000);
// assert(abbreviation_at(zone, summer) == "EDT");
// ```
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

// Converts an instant in an explicit zone, rejecting civil range overflow.
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// instant, _ := parse(Timestamp, "2024-03-10T07:30:00Z");
// wall := to_civil(zone, instant);
// assert(wall.time.hour == 3 && wall.time.minute == 30);
// ```
to_civil :: (zone: Time_Zone, instant: Timestamp) -> Date_Time {
    seconds := instant.seconds + offset_at(zone, instant).seconds;
    ns: s64 = instant.nanoseconds;

    if ns < 0 {
        seconds -= 1;
        ns += NS;
    }

    return .{ civil_day(floor_div(seconds, 86400)), clock_from_ns(modulo(seconds, 86400) * NS + ns) };
}

// Resolves gaps/folds using the requested policy and checks UTC range.
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// gap := Date_Time.{ .{ 2024, 3, 10 }, .{ 2, 30, 0, 0 } };
// instant, error := to_timestamp(zone, gap);
// assert(error.kind == .NONE && instant.seconds == 1710055800);
// _, rejected := to_timestamp(zone, gap, .REJECT);
// assert(rejected.kind == .GAP);
// ```
to_timestamp :: (zone: Time_Zone, wall: Date_Time, disambiguation := Disambiguation.COMPATIBLE) -> Timestamp, Time_Error {
    d, e := date(wall.date.year, wall.date.month, wall.date.day);
    if e.kind != .NONE return .{}, e;

    t, te := time_of_day(wall.time.hour, wall.time.minute, wall.time.second, wall.time.nanosecond);
    if te.kind != .NONE return .{}, te;

    local := date_days(d) * 86400 + time_ns(t) / NS;
    ns := time_ns(t) % NS;
    r: *Zone_Rules;

    if zone.kind == .RULES r = lookup_rules(zone.rules);

    offsets: [258] s64 = ---;
    count: s64 = 1;

    offsets[0] = zone.offset.seconds;
    if zone.kind == .UTC offsets[0] = 0;

    if r != null {
        // An instant that reads `local` lies within the largest offset of it, so only offsets in
        // effect during that window can match: the one at its start and those after each transition.
        window_end := local + 93599;
        cursor := raw_stamp(local - 93599, 0);

        offsets[0] = offset_at(zone, cursor).seconds;
        count = 1;

        while true {
            at, more := transition_after(r, cursor, false);
            if !more || at > window_end break;

            cursor = raw_stamp(at, 0);
            off := offset_at(zone, cursor).seconds;
            known := false;

            for 0..count - 1 if offsets[it] == off known = true;

            if !known {
                offsets[count] = off;
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

// Starts a borrowed cursor strictly after instant (strictly before when reverse).
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// start, _ := timestamp_from_seconds(1704067200);
// cursor := transitions(zone, start);
// change, found, error := next(*cursor);
// assert(error.kind == .NONE && found && change.after.seconds == -14400);
// ```
transitions :: (zone: Time_Zone, instant: Timestamp, reverse := false) -> Time_Transitions {
    return .{ zone, instant, reverse, false };
}

// Advances a transition cursor; false with no error means the range is exhausted.
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// start, _ := timestamp_from_seconds(1704067200);
// cursor := transitions(zone, start);
// spring, _, _ := next(*cursor);
// autumn, _, _ := next(*cursor);
// assert(spring.after.seconds == -14400 && autumn.after.seconds == -18000);
// ```
next :: (cursor: *Time_Transitions) -> Time_Transition, bool, Time_Error {
    if cursor.exhausted || cursor.zone.kind != .RULES || cursor.zone.rules == null return .{}, false, .{};

    r := lookup_rules(cursor.zone.rules);
    if r == null return .{}, false, .{};

    chosen, found := transition_after(r, cursor.cursor, cursor.reverse);

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

// Resolves an instant and borrows its zone; the zone owner must outlive the result.
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// instant, _ := timestamp_from_seconds(1720000000);
// z := zoned(instant, zone);
// assert(z.offset.seconds == -14400);
// ```
zoned :: (instant: Timestamp, zone: Time_Zone) -> Zoned {
    return .{ instant, zone, offset_at(zone, instant) };
}

// Resolves civil time and borrows the explicit zone.
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// fold := Date_Time.{ .{ 2024, 11, 3 }, .{ 1, 30, 0, 0 } };
// first, _ := to_zoned(fold, zone, .EARLIER);
// second, _ := to_zoned(fold, zone, .LATER);
// assert(first.offset.seconds == -14400 && second.offset.seconds == -18000);
// ```
to_zoned :: (wall: Date_Time, zone: Time_Zone, disambiguation := Disambiguation.COMPATIBLE) -> Zoned, Time_Error {
    t, e := to_timestamp(zone, wall, disambiguation);
    if e.kind != .NONE return .{}, e;
    return zoned(t, zone), .{};
}

// Reads the existing Basic consensus clock in an explicit borrowed zone.
//
// ```jai
// zone := time_zone_utc();
// z, error := zoned_now(zone);
// assert(error.kind == .NONE && z.offset.seconds == 0);
// ```
zoned_now :: (zone: Time_Zone) -> Zoned, Time_Error {
    t, e := timestamp_now();
    if e.kind != .NONE return .{}, e;
    return zoned(t, zone), .{};
}

// Keeps the instant while borrowing a different zone and resolving its offset.
//
// ```jai
// instant, _ := timestamp_from_seconds(0);
// o, _ := offset(32400);
// tokyo := with_zone(zoned(instant, time_zone_utc()), time_zone_fixed(o));
// assert(tokyo.timestamp.seconds == 0 && tokyo.offset.seconds == 32400);
// ```
with_zone :: (value: Zoned, zone: Time_Zone) -> Zoned {
    return .{ value.timestamp, zone, offset_at(zone, value.timestamp) };
}

// Adds calendar units in civil time, resolves, then adds clock units on the timeline.
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// start, _ := to_zoned(.{ .{ 2024, 3, 9 }, .{ 12, 0, 0, 0 } }, zone);
// by_day, _ := add(start, Span.{ days = 1 });
// by_hours, _ := add(start, Span.{ hours = 24 });
// assert(by_hours.timestamp.seconds - by_day.timestamp.seconds == 3600);
// ```
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

// Finds the first valid instant of the local date, including days starting after midnight.
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// noon, _ := to_zoned(.{ .{ 2024, 3, 10 }, .{ 15, 0, 0, 0 } }, zone);
// start, error := start_of_day(noon);
// assert(error.kind == .NONE && to_civil(zone, start.timestamp).time.hour == 0);
// ```
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

// Returns exact elapsed hours between consecutive local day starts.
//
// ```jai
// zone, zone_error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(zone_error.kind == .NONE);
// defer time_zone_free(*zone);
// day, _ := to_zoned(.{ .{ 2024, 3, 10 }, .{ 12, 0, 0, 0 } }, zone);
// hours, error := hours_in_day(day);
// assert(error.kind == .NONE && hours == 23);
// ```
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

// Records the latest of year's two rule events that is not after seconds.
posix_scan :: (p: Posix_Rules, year: s64, seconds: s64, last: *s64, dst: *bool) {
    start := rule_in_year(p.start, year, p.standard);
    end := rule_in_year(p.end, year, p.daylight);

    if start <= seconds && start > last.* {
        last.* = start;
        dst.* = true;
    }

    if end <= seconds && end > last.* {
        last.* = end;
        dst.* = false;
    }
}

posix_at :: (p: Posix_Rules, seconds: s64) -> s64, string {
    if !p.dst return p.standard, p.standard_name;

    year: s64 = civil_day(floor_div(seconds, 86400)).year;
    last: s64 = MIN_I64;
    dst := false;

    posix_scan(p, year, seconds, *last, *dst);

    // Extended transition times reach under ten days past a year's ends, so a neighbouring year
    // matters only when this one has no event yet or seconds is that close to its edge.
    reach := 10 * 86400;
    year_start := date_days(.{ cast,trunc(s16) year, 1, 1 }) * 86400;

    if last == MIN_I64 || seconds - year_start < reach posix_scan(p, year - 1, seconds, *last, *dst);
    if date_days(.{ cast,trunc(s16) (year + 1), 1, 1 }) * 86400 - seconds <= reach posix_scan(p, year + 1, seconds, *last, *dst);

    if dst return p.daylight, p.daylight_name;

    return p.standard, p.standard_name;
}

// The Unix second of the first transition strictly after `from` (strictly before when reverse):
// stored transitions first, then the footer rules beyond them.
transition_after :: (r: *Zone_Rules, from: Timestamp, reverse: bool) -> s64, bool {
    found := false;
    chosen: s64;
    lo: s64;
    hi := r.count;

    while lo < hi {
        mid := lo + (hi - lo) / 2;
        seconds := be_signed(r.source, r.ta + mid * r.width, r.width);
        c := compare(Timestamp.{ seconds, 0 }, from);

        if c < 0 || (!reverse && c == 0) lo = mid + 1;
        else hi = mid;
    }

    index := ifx reverse then lo - 1 else lo;

    if index >= 0 && index < r.count {
        chosen = be_signed(r.source, r.ta + index * r.width, r.width);
        found = true;
    }

    if !r.footer.present || !r.footer.dst return chosen, found;

    last := MIN_I64;
    if r.count > 0 last = be_signed(r.source, r.ta + (r.count - 1) * r.width, r.width);

    // Footer events all follow the last stored transition: a stored one ahead of the cursor
    // beats them, and none can precede a cursor that is not past the stored table.
    if reverse {
        if from.seconds <= last return chosen, found;
    } else if found return chosen, found;

    y: s64 = to_civil(time_zone_utc(), from).date.year;

    for year: y - 1..y + 1 {
        for event: s64.[rule_in_year(r.footer.start, year, r.footer.standard), rule_in_year(r.footer.end, year, r.footer.daylight)] {
            c := compare(Timestamp.{ event, 0 }, from);

            if event > last && ((reverse && c < 0) || (!reverse && c > 0)) && (!found || (reverse && event > chosen) || (!reverse && event < chosen)) {
                chosen = event;
                found = true;
            }
        }
    }

    return chosen, found;
}
