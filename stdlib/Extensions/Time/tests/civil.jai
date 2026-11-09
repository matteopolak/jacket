#import "Basic";
#import "String";
T :: #import "Extensions/Time";

civil_validation :: () {
    _, e := T.date(2023, 2, 29);

    assert(e.kind == .INVALID_VALUE);
    d, ok := T.date(2024, 2, 29);

    assert(ok.kind == .NONE);
    assert(T.day_of_year(d) == 60 && T.weekday(d) == .THURSDAY);
    y, w := T.iso_week(T.Date.{ 2021, 1, 1 });

    assert(y == 2020 && w == 53);
    _, bad := T.date_from_iso_week(2021, 53, .MONDAY);

    assert(bad.kind != .NONE);
    last := T.last_of_month(T.Date.{ 2024, 2, 1 });

    assert(last.day == 29);
    T.last_of_month(T.Date.{ 2024, 127, 1 }); // Invalid literals remain memory-safe.
} @test

civil_arithmetic :: () {
    d, e := T.add(T.Date.{ 2024, 1, 31 }, T.Span.{ months = 1 });

    assert(e.kind == .NONE && d.day == 29);
    _, reject := T.add(T.Date.{ 2024, 1, 31 }, T.Span.{ months = 1 }, .REJECT);

    assert(reject.kind != .NONE);
    t, te := T.add(T.Time_Of_Day.{ 23, 30, 0, 0 }, T.Span.{ hours = 25 });

    assert(te.kind == .NONE && t.hour == 0 && t.minute == 30);
    dt, de := T.add(T.Date_Time.{ .{ 2024, 2, 28 }, .{ 23, 0, 0, 0 } }, T.Span.{ hours = 2 });

    assert(de.kind == .NONE && dt.date.day == 29 && dt.time.hour == 1);

    for year: -100..100 {
        a := T.Date.{ cast(s16) year, 1, 31 };
        b := T.Date.{ cast(s16) (year + 1), 3, 1 };
        diff, error := T.until(a, b);

        assert(error.kind == .NONE);
        result, error2 := T.add(a, diff);

        assert(error2.kind == .NONE && T.equal(result, b));
    }
} @test

civil_text :: () {
    d, e := T.parse(T.Date, "2024-02-29");

    assert(e.kind == .NONE && d.day == 29);
    _, bad := T.parse(T.Date, "2024/02-29");

    assert(bad.kind == .SYNTAX && bad.offset == 4);
    dt, de := T.parse(T.Date_Time, "-009999-01-01t02:30:00.123456789");

    assert(de.kind == .NONE && dt.date.year == -9999);
    builder: String_Builder;

    defer free_buffers(*builder);
    error := T.format(*builder, dt);

    assert(error.kind == .NONE);
    text := builder_to_string(*builder);

    defer free(text);
    copy, ce := T.parse(T.Date_Time, text);

    assert(ce.kind == .NONE && T.equal(copy, dt));

    for n: 0..21 {
        _, cut := T.parse(T.Date_Time, slice(text, 0, n));
        assert(cut.kind != .NONE);
    }
} @test

civil_random_roundtrip :: () #no_aoc {
    seed: u64 = 42;

    for 1..1000 {
        seed = seed * 6364136223846793005 + 1;
        year := cast(s64) (seed % 19999) - 9999;
        month := cast(s64) ((seed >> 16) % 12) + 1;
        day := cast(s64) ((seed >> 32) % 28) + 1;
        d, e := T.date(year, month, day);

        assert(e.kind == .NONE);
        b: String_Builder;
        fe := T.format(*b, d);

        assert(fe.kind == .NONE);
        text := builder_to_string(*b);

        defer free(text);
        copy, ce := T.parse(T.Date, text);

        assert(ce.kind == .NONE && T.equal(d, copy));
    }
} @test

// Calendar examples: https://docs.rs/jiff/latest/jiff/civil/struct.Date.html#method.checked_add
// Jiff's Date + clock-unit examples are outside this API: Date explicitly rejects clock units.
// Temporal examples: https://tc39.es/proposal-temporal/docs/plaindate.html#dateaddduration-temporalduration--object--string-options-object--temporalplaindate
published_calendar_examples :: () {
    for months, i: s64.[1, 2, -1] {
        expected := T.Date.[.{ 2024, 4, 30 }, .{ 2024, 5, 31 }, .{ 2024, 2, 29 }];
        got, e := T.add(T.Date.{ 2024, 3, 31 }, T.Span.{ months = cast(s32) months });
        assert(e.kind == .NONE && T.equal(got, expected[i]));
    }

    got, e := T.add(T.Date.{ 2006, 8, 24 }, T.Span.{ years = 20, months = 4 });

    assert(e.kind == .NONE && T.equal(got, T.Date.{ 2026, 12, 24 }));
    diff, de := T.until(T.Date.{ 2006, 8, 24 }, T.Date.{ 2019, 1, 31 }, .YEAR);

    assert(de.kind == .NONE && diff.years == 12 && diff.months == 5 && diff.days == 7);
    days, dye := T.until(T.Date.{ 2006, 8, 24 }, T.Date.{ 2019, 1, 31 });

    assert(dye.kind == .NONE && days.days == 4543);
} @test
