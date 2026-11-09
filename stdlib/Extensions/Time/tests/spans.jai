#import "Basic";
ST :: #import "Extensions/Time";

span_rounding :: () {
    modes := ST.Rounding.[.HALF_EXPAND, .TRUNC, .FLOOR, .CEIL, .EXPAND, .HALF_TRUNC, .HALF_EVEN, .HALF_FLOOR, .HALF_CEIL];
    positive := s64.[3, 2, 2, 3, 3, 2, 2, 2, 3];
    negative := s64.[-3, -2, -3, -2, -3, -2, -2, -3, -2];

    for mode, i: modes {
        p, e := ST.span_round(.{ seconds = 2, milliseconds = 500 }, .SECOND, .SECOND, mode);
        assert(e.kind == .NONE && p.seconds == positive[i]);
        n, ne := ST.span_round(.{ seconds = -2, milliseconds = -500 }, .SECOND, .SECOND, mode);
        assert(ne.kind == .NONE && n.seconds == negative[i]);
    }

    _, mixed := ST.span_check(.{ years = 1, days = -1 });

    assert(mixed.kind == .INVALID_VALUE);
    _, relative := ST.span_total(.{ months = 1 }, .DAY);

    assert(relative.kind == .RELATIVE_REQUIRED);
    n, e := ST.span_total(.{ months = 1 }, .DAY, ST.Date.{ 2024, 2, 1 });

    assert(e.kind == .NONE && n == 29);
    balanced, be := ST.span_balance(.{ hours = 25 }, .DAY);

    assert(be.kind == .NONE && balanced.days == 1 && balanced.hours == 1);
} @test

calendar_differences :: () {
    for year: -20..20 {
        a := ST.Date.{ cast(s16) year, 1, 31 };
        b := ST.Date.{ cast(s16) (year + 1), 3, 30 };

        for largest: ST.Unit.[.YEAR, .MONTH, .WEEK, .DAY] {
            s, e := ST.until(a, b, largest);

            assert(e.kind == .NONE);
            r, re := ST.add(a, s);

            assert(re.kind == .NONE && ST.equal(r, b));
            back, be := ST.until(b, a, largest);

            assert(be.kind == .NONE);
            r2, r2e := ST.add(b, back);

            assert(r2e.kind == .NONE && ST.equal(r2, a));
        }
    }

    a := ST.Date_Time.{ .{ 2024, 2, 1 }, .{ 12, 0, 0, 0 } };
    b := ST.Date_Time.{ .{ 2024, 2, 3 }, .{ 1, 0, 0, 0 } };
    s, e := ST.until(a, b);

    assert(e.kind == .NONE && s.days == 1 && s.hours == 13);
    n, ne := ST.until(a, b, .DAY, .DAY, .HALF_EXPAND);

    assert(ne.kind == .NONE && n.days == 2);
} @test

span_text :: () {
    s, e := ST.parse(ST.Span, "-P1Y2M3W4DT5H6M7.008009010S");

    assert(e.kind == .NONE && s.years == -1 && s.nanoseconds == -10);
    b: String_Builder;

    defer free_buffers(*b);
    fe := ST.format(*b, s);

    assert(fe.kind == .NONE);
    text := builder_to_string(*b);

    defer free(text);
    assert(text == "-P1Y2M3W4DT5H6M7.00800901S");

    for text: string.["P", "PT", "P1D1Y", "P1Y1Y", "PT1.1234567890S", "P-1D", "P1DT"] {
        _, bad := ST.parse(ST.Span, text);
        assert(bad.kind != .NONE);
    }
} @test

span_large_units :: () {
    v, e := ST.span_balance(.{ hours = 9223372036854775807 }, .HOUR);

    assert(e.kind == .NONE && v.hours == 9223372036854775807);
    v2, e2 := ST.span_check(.{ seconds = -2147483648 });

    assert(e2.kind == .NONE && v2.seconds == -2147483648);

    for text: string.["PT1", "PT999999999999999999999999S", "PT1.5H"] {
        _, bad := ST.parse(ST.Span, text);
        assert(bad.kind != .NONE && bad.offset >= 0 && bad.offset <= text.count, tprint("%: % at %", text, bad.kind, bad.offset));
    }
} @test

span_zoned_relative :: () {
    zone, e := ST.time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");

    assert(e.kind == .NONE);
    defer ST.time_zone_free(*zone);
    start, se := ST.to_zoned(.{ .{ 2024, 3, 9 }, .{ 12, 0, 0, 0 } }, zone);

    assert(se.kind == .NONE);
    hours, he := ST.span_total(.{ days = 1 }, .HOUR, start);

    assert(he.kind == .NONE && hours == 23);
    end, ee := ST.add(start, ST.Span.{ days = 1, hours = 2 });

    assert(ee.kind == .NONE);
    diff, de := ST.until(start, end);

    assert(de.kind == .NONE && diff.days == 1 && diff.hours == 2);
    back, be := ST.add(start, diff);

    assert(be.kind == .NONE && ST.equal(back, end));
} @test

// Rounding to days measures the neighbouring days with the caller's gap policy: with .EARLIER
// the day after 02:30 on 2024-03-09 starts an hour sooner than with the default, so the same
// end is past the midpoint instead of exactly on it.
rounding_follows_disambiguation :: () {
    zone, ze := ST.time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");

    assert(ze.kind == .NONE);
    defer ST.time_zone_free(*zone);
    a, ae := ST.to_zoned(.{ .{ 2024, 3, 9 }, .{ 2, 30, 0, 0 } }, zone);
    b, be := ST.to_zoned(.{ .{ 2024, 3, 10 }, .{ 15, 0, 0, 0 } }, zone);

    assert(ae.kind == .NONE && be.kind == .NONE);
    tie, te := ST.until(a, b, .DAY, .DAY, .HALF_TRUNC, 1, .COMPATIBLE);
    early, ee := ST.until(a, b, .DAY, .DAY, .HALF_TRUNC, 1, .EARLIER);

    assert(te.kind == .NONE && ee.kind == .NONE);
    assert(tie.days == 1 && early.days == 2);
} @test
