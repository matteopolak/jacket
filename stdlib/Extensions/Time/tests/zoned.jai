#import "Basic";
ZT :: #import "Extensions/Time";

zoned_dst :: () {
    zone, e := ZT.time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");

    assert(e.kind == .NONE);
    defer ZT.time_zone_free(*zone);
    wall := ZT.Date_Time.{ .{ 2024, 3, 10 }, .{ 2, 30, 0, 0 } };
    early, ee := ZT.to_timestamp(zone, wall, .EARLIER);
    later, le := ZT.to_timestamp(zone, wall, .LATER);

    assert(ee.kind == .NONE && le.kind == .NONE && later.seconds - early.seconds == 3600);
    assert(ZT.to_civil(zone, early).time.hour == 1 && ZT.to_civil(zone, later).time.hour == 3);
    _, reject := ZT.to_timestamp(zone, wall, .REJECT);

    assert(reject.kind == .GAP);
    fold := ZT.Date_Time.{ .{ 2024, 11, 3 }, .{ 1, 30, 0, 0 } };
    a, ae := ZT.to_timestamp(zone, fold);
    b, be := ZT.to_timestamp(zone, fold, .LATER);

    assert(ae.kind == .NONE && be.kind == .NONE && b.seconds - a.seconds == 3600);
    start, se := ZT.to_zoned(.{ .{ 2024, 3, 9 }, .{ 12, 0, 0, 0 } }, zone);

    assert(se.kind == .NONE);
    day, de := ZT.add(start, ZT.Span.{ days = 1 });
    hours, he := ZT.add(start, ZT.Span.{ hours = 24 });

    assert(de.kind == .NONE && he.kind == .NONE && hours.timestamp.seconds - day.timestamp.seconds == 3600);
    n, ne := ZT.hours_in_day(day);

    assert(ne.kind == .NONE && n == 23);
    cursor := ZT.transitions(zone, early);
    change, found, ce := ZT.next(*cursor);

    assert(found && ce.kind == .NONE && change.after.seconds == -14400);
} @test

zoned_iso :: () {
    z, e := ZT.parse(ZT.Zoned, "2024-03-10T03:30:00-04:00[-04:00]");

    assert(e.kind == .NONE);
    defer ZT.time_zone_free(*z.zone);
    b: String_Builder;

    defer free_buffers(*b);
    fe := ZT.format(*b, z);

    assert(fe.kind == .NONE);
    text := builder_to_string(*b);

    defer free(text);
    assert(text == "2024-03-10T03:30:00-04:00[-04:00]");
    _, conflict := ZT.parse(ZT.Zoned, "2024-03-10T03:30:00-05:00[-04:00]");

    assert(conflict.kind == .OFFSET_CONFLICT);
    bare, be := ZT.parse(ZT.Zoned, "2024-03-10T03:30:00[+05:30]");

    assert(be.kind == .NONE && bare.offset.seconds == 19800);
} @test

ZEnv :: #import "Extensions/Environment";

zoned_historical_days :: () {
    root := ZEnv.output_directory();

    for name: string.["America/Sao_Paulo", "Asia/Gaza", "Pacific/Apia"] {
        zone, e := ZT.time_zone_get(name, directory = tprint("%/time-conformance/zoneinfo", root));

        assert(e.kind == .NONE, tprint("%: % (build the zone data: `PYTHONPATH=tools python3 tools/portable/build_portable.py data`)", name, e.message));
        defer ZT.time_zone_free(*zone);
        wall := ZT.Date_Time.{ .{ 2018, 11, 4 }, .{ 12, 0, 0, 0 } };
        if name == "Asia/Gaza" wall.date = .{ 2020, 3, 28 };
        if name == "Pacific/Apia" wall.date = .{ 2011, 12, 31 };

        z, ze := ZT.to_zoned(wall, zone);

        assert(ze.kind == .NONE);
        start, se := ZT.start_of_day(z);

        assert(se.kind == .NONE);
        civil := ZT.to_civil(zone, start.timestamp);

        assert(civil.time.hour == (ifx name == "Pacific/Apia" then 0 else 1));

        if name == "Pacific/Apia" {
            _, reject := ZT.to_timestamp(zone, .{ .{ 2011, 12, 30 }, .{} }, .REJECT);
            assert(reject.kind == .GAP);
        }
    }
} @test
