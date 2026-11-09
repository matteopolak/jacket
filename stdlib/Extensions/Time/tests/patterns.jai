#import "Basic";
PT :: #import "Extensions/Time";

pattern_formatting :: () {
    wall := PT.Date_Time.{ .{ 2024, 2, 29 }, .{ 15, 4, 5, 123000000 } };
    b: String_Builder;

    defer free_buffers(*b);
    e := PT.format(*b, wall, "%Y-%m-%d %A %b %j %G-W%V-%u %I:%M:%S %p %.f %f %_d %-m %%");

    assert(e.kind == .NONE);
    text := builder_to_string(*b);

    defer free(text);
    assert(text == "2024-02-29 Thursday Feb 060 2024-W09-4 03:04:05 PM .123 123000000 29 2 %", text);
    parsed, pe := PT.parse_format(PT.Date_Time, "%Y-%m-%d %H:%M:%S%.f", "2024-02-29 15:04:05.123");

    assert(pe.kind == .NONE && PT.equal(parsed, wall));
    iso, ie := PT.parse_format(PT.Date, "%G-W%V-%u", "2020-W53-5");

    assert(ie.kind == .NONE && iso.year == 2021 && iso.month == 1 && iso.day == 1);
    ordinal, oe := PT.parse_format(PT.Date, "%Y %j", "2024 060");

    assert(oe.kind == .NONE && ordinal.month == 2 && ordinal.day == 29);
    _, conflict := PT.parse_format(PT.Date, "%Y-%m-%d %j", "2024-02-29 061");

    assert(conflict.kind != .NONE);
    _, missing := PT.parse_format(PT.Date, "%m-%d", "02-29");

    assert(missing.kind != .NONE);
    t, te := PT.parse_format(PT.Timestamp, "%Y-%m-%d %H:%M:%S %:z", "2024-02-29 15:04:05 +05:30");

    assert(te.kind == .NONE && PT.to_civil(PT.time_zone_fixed(.{ 19800 }), t).time.hour == 15);
} @test

pattern_negative_and_clock :: () {
    b: String_Builder;

    defer free_buffers(*b);
    e := PT.format(*b, PT.Date.{ -1, 1, 1 }, "%Y");

    assert(e.kind == .NONE);
    text := builder_to_string(*b);

    defer free(text);
    assert(text == "-0001", text);
    clock, ce := PT.parse_format(PT.Time_Of_Day, "%I:%M %p", "01:30 PM");

    assert(ce.kind == .NONE && clock.hour == 13);
    _, bad := PT.parse_format(PT.Date, "%G-W%V-%-u", "2024-W01-9999");

    assert(bad.kind != .NONE);
} @test

pattern_conflicts_and_epoch :: () {
    _, bad := PT.parse_format(PT.Timestamp, "%s %Y-%m-%d %H:%M", "0 1970-01-01 01:00");

    assert(bad.kind != .NONE);
    _, weekday := PT.parse_format(PT.Date, "%Y-%m-%d %u", "2024-02-29 5");

    assert(weekday.kind != .NONE);
    _, year := PT.parse_format(PT.Date, "%Y %G-W%V-%u", "2023 2024-W01-1");

    assert(year.kind != .NONE);
    t, e := PT.parse_format(PT.Timestamp, "%s%.f", "-1.5");

    assert(e.kind == .NONE && t.seconds == 0 && t.nanoseconds == -500000000);
} @test

space_is_not_a_directive :: () {
    clock := PT.Time_Of_Day.{ 1, 2, 3, 0 };
    b: String_Builder;

    defer free_buffers(*b);
    e := PT.format(*b, clock, "% ");

    assert(e.kind == .SYNTAX && e.message == "unknown pattern directive");
    _, pe := PT.parse_format(PT.Date, "% ", "2024-01-01");

    assert(pe.kind == .SYNTAX && pe.message == "unknown pattern directive");
} @test
