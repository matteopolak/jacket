#import "Basic";
Exact :: #import "Extensions/Time";

exact_normalization :: () {
    d, e := Exact.signed_duration(1, -1);

    assert(e.kind == .NONE && d.seconds == 0 && d.nanoseconds == 999999999);
    _, bad := Exact.signed_duration(9223372036854775807, 1000000000);

    assert(bad.kind != .NONE);
    t, te := Exact.timestamp_from_millis(-1);

    assert(te.kind == .NONE && t.seconds == 0 && t.nanoseconds == -1000000);
    wall := Exact.to_civil(Exact.time_zone_utc(), t);

    assert(wall.date.year == 1969 && wall.time.second == 59 && wall.time.nanosecond == 999000000);
    copy, ce := Exact.to_timestamp(Exact.time_zone_utc(), wall);

    assert(ce.kind == .NONE && Exact.equal(t, copy));
    _, min_error := Exact.to_timestamp(Exact.time_zone_utc(), Exact.Date_Time.{ .{ -9999, 1, 1 }, .{} });

    assert(min_error.kind == .OUT_OF_RANGE);
    min, me := Exact.to_timestamp(Exact.time_zone_utc(), Exact.Date_Time.{ .{ -9999, 1, 2 }, .{ 1, 59, 59, 0 } });

    assert(me.kind == .NONE);
    o, oe := Exact.offset(-93599);

    assert(oe.kind == .NONE);
    edge := Exact.to_civil(Exact.time_zone_fixed(o), min);

    assert(edge.date.year == -9999 && edge.date.day == 1);
    plus, pe := Exact.add(t, Exact.Signed_Duration.{ 1, 1 });

    assert(pe.kind == .NONE && plus.seconds == 0 && plus.nanoseconds == 999000001);
    diff := Exact.duration_between(t, plus);

    assert(diff.seconds == 1 && diff.nanoseconds == 1);
} @test

exact_text :: () {
    t, e := Exact.parse(Exact.Timestamp, "1985-04-12t23:20:50.52+02:30");

    assert(e.kind == .NONE);
    b: String_Builder;

    defer free_buffers(*b);
    fe := Exact.format(*b, t);

    assert(fe.kind == .NONE);
    text := builder_to_string(*b);

    defer free(text);
    assert(text == "1985-04-12T20:50:50.52Z");
    copy, ce := Exact.parse(Exact.Timestamp, text);

    assert(ce.kind == .NONE && Exact.equal(t, copy));
    _, leap := Exact.parse(Exact.Timestamp, "1990-12-31T23:59:60Z");

    assert(leap.kind != .NONE);
    _, tail := Exact.parse(Exact.Timestamp, "2000-01-01T00:00:00Zjunk");

    assert(tail.kind == .SYNTAX && tail.offset == 20);
    _, zero := Exact.parse(Exact.Offset, "+26:00");

    assert(zero.kind != .NONE);
    now, ne := Exact.timestamp_now();

    assert(ne.kind == .NONE && now.seconds > 1700000000);
} @test

exact_extreme_duration :: () {
    for v: Exact.Signed_Duration.[.{ -9223372036854775808, -999999999 }, .{ 9223372036854775807, 999999999 }, .{ 0, -1 }] {
        b: String_Builder;

        defer free_buffers(*b);
        e := Exact.format(*b, v);

        assert(e.kind == .NONE);
        text := builder_to_string(*b);

        defer free(text);
        copy, ce := Exact.parse(Exact.Signed_Duration, text);

        assert(ce.kind == .NONE && Exact.equal(v, copy), text);
    }
} @test
