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
