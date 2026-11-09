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

invalid_literals_remain_bounded :: () {
    Prop.to_civil(Prop.time_zone_utc(), .{ 9223372036854775807, 0 });
    Prop.to_civil(Prop.time_zone_utc(), .{ -9223372036854775808, 0 });
    Prop.iso_week(.{ 32767, 12, 31 });
    Prop.iso_week(.{ -32768, 1, 1 });
    _, bad := Prop.parse(Prop.Date, "2024-01-01", .{ max_transitions = -1 });

    assert(bad.kind == .LIMIT);
} @test
