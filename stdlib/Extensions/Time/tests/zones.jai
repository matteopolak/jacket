#import "Basic";
Zone_Test :: #import "Extensions/Time";

posix_rules :: () {
    z, e := Zone_Test.time_zone_from_posix("EST5EDT,M3.2.0/2,M11.1.0/2");

    assert(e.kind == .NONE);
    defer Zone_Test.time_zone_free(*z);
    winter, _ := Zone_Test.parse(Zone_Test.Timestamp, "2024-01-01T00:00:00Z");
    summer, _ := Zone_Test.parse(Zone_Test.Timestamp, "2024-07-01T00:00:00Z");

    assert(Zone_Test.offset_at(z, winter).seconds == -18000 && Zone_Test.offset_at(z, summer).seconds == -14400);
    assert(Zone_Test.abbreviation_at(z, summer) == "EDT");
    retained := Zone_Test.time_zone_retain(z);

    Zone_Test.time_zone_free(*z);
    assert(Zone_Test.offset_at(retained, summer).seconds == -14400);
    Zone_Test.time_zone_free(*retained);

    for text: string.["", "EST", "EST26", "EST5EDT,M13.1.0,M11.1.0", "<x>0", "EST5EDT,J0,J366", "EST5EDT,366,1", "EST5EDT,M3.2.0/168,M11.1.0"] {
        _, bad := Zone_Test.time_zone_from_posix(text);
        assert(bad.kind != .NONE);
    }

    z2, e2 := Zone_Test.time_zone_from_posix("<+03>-3");

    assert(e2.kind == .NONE);
    defer Zone_Test.time_zone_free(*z2);
    assert(Zone_Test.offset_at(z2, winter).seconds == 10800);
} @test

malformed_tzif :: () {
    for n: 0..80 {
        bytes: [80] u8;
        _, error := Zone_Test.time_zone_from_tzif("", .{ n, bytes.data });
        assert(error.kind != .NONE);
    }

    for name: string.["../UTC", "/UTC", "a/../UTC", "a\\UTC", "a//b", "./UTC"] {
        _, error := Zone_Test.time_zone_get(name);
        assert(error.kind != .NONE);
    }
} @test

forged_zone_handle :: () {
    bad := Zone_Test.Time_Zone.{ .RULES, .{}, cast(*void) 1 };
    assert(Zone_Test.offset_at(bad, .{}).seconds == 0);
    Zone_Test.abbreviation_at(bad, .{});
    Zone_Test.time_zone_free(*bad);
} @test

stale_zone_handle :: () {
    zone, e := Zone_Test.time_zone_from_posix("AAA5");

    assert(e.kind == .NONE);
    assert(Zone_Test.offset_at(zone, .{}).seconds == -18000);
    stale := zone;
    other := Zone_Test.time_zone_retain(zone);

    Zone_Test.time_zone_free(*zone);
    assert(Zone_Test.offset_at(stale, .{}).seconds == -18000);
    Zone_Test.time_zone_free(*other);

    // The handle was valid and recently used, but its rules are gone: it must read as unknown.
    assert(Zone_Test.offset_at(stale, .{}).seconds == 0);
    assert(Zone_Test.abbreviation_at(stale, .{}) == "");
} @test

Edge_Reading :: struct {
    text: string;
    offset: s64;
}

// Daylight time starts a week before the new year (J1/-167) and ends in July, so the event that
// governs a late-December reading belongs to the following year's rule.
posix_events_across_year_edges :: () {
    zone, e := Zone_Test.time_zone_from_posix("AAA0BBB,J1/-167,J200/0");

    assert(e.kind == .NONE);
    defer Zone_Test.time_zone_free(*zone);

    for expected: Edge_Reading.[
        .{ "2025-12-24T12:00:00Z", 0 },
        .{ "2025-12-25T00:59:59Z", 0 },
        .{ "2025-12-25T01:00:00Z", 3600 },
        .{ "2025-12-28T12:00:00Z", 3600 },
        .{ "2026-01-03T12:00:00Z", 3600 },
        .{ "2026-07-18T22:59:59Z", 3600 },
        .{ "2026-07-18T23:00:00Z", 0 },
        .{ "2026-10-01T00:00:00Z", 0 }
    ] {
        instant, pe := Zone_Test.parse(Zone_Test.Timestamp, expected.text);

        assert(pe.kind == .NONE);
        assert(Zone_Test.offset_at(zone, instant).seconds == expected.offset);
    }
} @test

posix_julian_and_extended :: () {
    for rules, index: string.["AAA0BBB,J60/0,J300/0", "AAA0BBB,59/0,300/0"] {
        zone, e := Zone_Test.time_zone_from_posix(rules);

        assert(e.kind == .NONE);
        defer Zone_Test.time_zone_free(*zone);
        leap, _ := Zone_Test.parse(Zone_Test.Timestamp, "2024-02-29T12:00:00Z");
        march, _ := Zone_Test.parse(Zone_Test.Timestamp, "2024-03-01T12:00:00Z");

        assert(Zone_Test.offset_at(zone, leap).seconds == (ifx index == 0 then 0 else 3600));
        assert(Zone_Test.offset_at(zone, march).seconds == 3600);
    }

    zone, e := Zone_Test.time_zone_from_posix("AAA0BBB,M3.1.0/-2,M11.1.0/26");

    assert(e.kind == .NONE);
    defer Zone_Test.time_zone_free(*zone);
    spring, _ := Zone_Test.parse(Zone_Test.Timestamp, "2024-03-02T22:30:00Z");
    fall, _ := Zone_Test.parse(Zone_Test.Timestamp, "2024-11-04T01:00:00Z");

    assert(Zone_Test.offset_at(zone, spring).seconds == 3600 && Zone_Test.offset_at(zone, fall).seconds == 0);
} @test

all_tzif_versions :: () {
    header: [54] u8;
    header[0] = 84;
    header[1] = 90;
    header[2] = 105;
    header[3] = 102;
    header[39] = 1;
    header[43] = 4;
    header[50] = 85;
    header[51] = 84;
    header[52] = 67;
    first, e := Zone_Test.time_zone_from_tzif("", header);

    assert(e.kind == .NONE);
    Zone_Test.time_zone_free(*first);

    for version: 50..52 {
        bytes: [114] u8;

        for c, i: header {
            bytes[i] = c;
            bytes[54 + i] = c;
        }

        bytes[4] = cast(u8) version;
        bytes[58] = cast(u8) version;
        bytes[108] = 10;
        bytes[109] = 85;
        bytes[110] = 84;
        bytes[111] = 67;
        bytes[112] = 48;
        bytes[113] = 10;
        zone, ze := Zone_Test.time_zone_from_tzif("Example/Zone", bytes);

        assert(ze.kind == .NONE);
        Zone_Test.time_zone_free(*zone);

        for cut: 0..113 {
            _, bad := Zone_Test.time_zone_from_tzif("", .{ cut, bytes.data });
            assert(bad.kind != .NONE);
        }
    }
} @test
