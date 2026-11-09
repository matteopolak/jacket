#import "Basic";
Prop :: #import "Extensions/Time";

random_timestamp_text :: () #no_aoc {
    seed: u64 = 1234;

    for 1..1000 {
        seed = seed * 6364136223846793005 + 1;
        seconds := cast(s64) (seed % 600000000000) - 300000000000;
        t, e := Prop.timestamp_from_seconds(seconds);

        if e.kind != .NONE continue;

        b: String_Builder;
        fe := Prop.format(*b, t);

        assert(fe.kind == .NONE);
        text := builder_to_string(*b);

        defer free(text);
        copy, ce := Prop.parse(Prop.Timestamp, text);

        assert(ce.kind == .NONE && Prop.equal(t, copy));
        assert(Prop.hash(t) == Prop.hash(copy));
    }
} @test

bounded_malformed_text :: () #no_aoc {
    seed: u64 = 9;

    for 1..2000 {
        bytes: [40] u8;
        for *c: bytes {
            seed = seed * 6364136223846793005 + 1;
            c.* = cast(u8) (seed >> 56);
        }

        text := string.{ cast(s64) (seed % 40), bytes.data };

        Prop.parse(Prop.Date, text);
        Prop.parse(Prop.Time_Of_Day, text);
        Prop.parse(Prop.Date_Time, text);
        Prop.parse(Prop.Timestamp, text);
        Prop.parse(Prop.Span, text);
        Prop.parse(Prop.Signed_Duration, text);
        Prop.parse_format(Prop.Date_Time, "%Y-%m-%d %H:%M:%S", text);
    }

    _, limit := Prop.parse(Prop.Date, "2024-01-01", .{ max_text = 3 });

    assert(limit.kind == .LIMIT);
} @test

span_api_overloads :: () {
    origin := Prop.Date.{ 2024, 1, 1 };
    zone := Prop.time_zone_utc();
    z, ze := Prop.to_zoned(.{ origin, .{} }, zone);

    assert(ze.kind == .NONE);
    b1, e1 := Prop.span_balance(.{ months = 1 }, .DAY, origin);
    b2, e2 := Prop.span_balance(.{ months = 1 }, .DAY, z);

    assert(e1.kind == .NONE && e2.kind == .NONE && b1.days == 31 && b2.days == 31);
    r1, re1 := Prop.span_round(.{ months = 1 }, .DAY, .DAY, relative = origin);
    r2, re2 := Prop.span_round(.{ months = 1 }, .DAY, .DAY, relative = z);

    assert(re1.kind == .NONE && re2.kind == .NONE && r1.days == 31 && r2.days == 31);
    d1, de1 := Prop.span_to_duration(.{ months = 1 }, origin);
    d2, de2 := Prop.span_to_duration(.{ months = 1 }, z);

    assert(de1.kind == .NONE && de2.kind == .NONE && d1.seconds == 31 * 86400 && d2.seconds == 31 * 86400);
    c1, ce1 := Prop.span_compare(.{ months = 1 }, .{ days = 31 }, origin);
    c2, ce2 := Prop.span_compare(.{ months = 1 }, .{ days = 31 }, z);

    assert(ce1.kind == .NONE && ce2.kind == .NONE && c1 == 0 && c2 == 0);
    instant, _ := Prop.timestamp_from_seconds(0);
    end, _ := Prop.timestamp_from_seconds(2);
    diff, e := Prop.until(instant, end);

    assert(e.kind == .NONE && diff.seconds == 2);
    back, be := Prop.since(end, instant);

    assert(be.kind == .NONE && back.seconds == 2);
    assert(Prop.hash(zone) == Prop.hash(Prop.time_zone_utc()) && Prop.equal(z, Prop.with_zone(z, Prop.time_zone_fixed(.{ 3600 }))));
} @test

random_wall :: (seed: *u64) -> Prop.Date_Time #no_aoc {
    seed.* = seed.* * 6364136223846793005 + 1;
    r := seed.* >> 16;

    year: s64 = cast(s64) (r % 301) + 1900;

    if ((r >> 20) % 5) == 0 year = cast(s64) ((r >> 24) % 19999) - 9999;

    month := cast(s64) ((r >> 8) % 12) + 1;
    last, _ := Prop.days_in_month(year, month);
    day := last - cast(s64) ((r >> 12) % 4);

    if ((r >> 14) % 3) == 0 day = cast(s64) ((r >> 30) % cast(u64) last) + 1;

    seed.* = seed.* * 6364136223846793005 + 1;
    q := seed.* >> 16;
    wall, e := Prop.date_time(year, month, day, cast(s64) (q % 24), cast(s64) ((q >> 8) % 60), cast(s64) ((q >> 16) % 60), cast(s64) ((q >> 24) % 1000000000));

    assert(e.kind == .NONE);

    return wall;
}

// Whatever the span between two readings, adding it back must land exactly on the end, and
// one more of the coarsest unit must overshoot it.
random_calendar_differences :: () #no_aoc {
    seed: u64 = 77;

    for 1..1500 {
        a := random_wall(*seed);
        b := random_wall(*seed);
        sign := Prop.compare(b, a);

        for largest: Prop.Unit.[.YEAR, .MONTH, .WEEK, .DAY] {
            s, e := Prop.until(a, b, largest);

            assert(e.kind == .NONE);
            _, ce := Prop.span_check(s);
            back, be := Prop.add(a, s);

            assert(ce.kind == .NONE && be.kind == .NONE && Prop.equal(back, b));

            more: Prop.Span;

            if largest == .YEAR more.years = s.years + cast(s32) sign;
            else if largest == .MONTH more.months = s.months + cast(s32) sign;
            else if largest == .WEEK more.weeks = s.weeks + cast(s32) sign;
            else more.days = s.days + cast(s32) sign;

            over, oe := Prop.add(a, more);

            assert(sign == 0 || oe.kind != .NONE || Prop.compare(over, b) * sign > 0);
        }
    }
} @test

// The same bracketing in a zone with gaps and folds, where the end is only guaranteed not to be passed.
random_zoned_differences :: () #no_aoc {
    zone, ze := Prop.time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");

    assert(ze.kind == .NONE);
    defer Prop.time_zone_free(*zone);
    seed: u64 = 5;

    for 1..1500 {
        a, ae := Prop.to_zoned(random_wall(*seed), zone);
        b, be := Prop.to_zoned(random_wall(*seed), zone);

        if ae.kind != .NONE || be.kind != .NONE continue;

        sign := Prop.compare(b, a);

        for largest: Prop.Unit.[.YEAR, .MONTH, .WEEK, .DAY] {
            s, e := Prop.until(a, b, largest);

            assert(e.kind == .NONE);
            _, ce := Prop.span_check(s);
            reached, re := Prop.add(a, s);

            assert(ce.kind == .NONE && re.kind == .NONE && Prop.compare(reached, b) * sign <= 0);

            more: Prop.Span;

            if largest == .YEAR more.years = s.years + cast(s32) sign;
            else if largest == .MONTH more.months = s.months + cast(s32) sign;
            else if largest == .WEEK more.weeks = s.weeks + cast(s32) sign;
            else more.days = s.days + cast(s32) sign;

            over, oe := Prop.add(a, more);

            assert(sign == 0 || oe.kind != .NONE || Prop.compare(over, b) * sign > 0);
        }
    }
} @test

invalid_literals_remain_bounded :: () {
    Prop.to_civil(Prop.time_zone_utc(), .{ 9223372036854775807, 0 });
    Prop.to_civil(Prop.time_zone_utc(), .{ -9223372036854775808, 0 });
    Prop.iso_week(.{ 32767, 12, 31 });
    Prop.iso_week(.{ -32768, 1, 1 });
    _, bad := Prop.parse(Prop.Date, "2024-01-01", .{ max_transitions = -1 });

    assert(bad.kind == .LIMIT);
} @test
