#import "Basic";
IT :: #import "Extensions/Time";
IJ :: #import "Extensions/Json";

interop_basic :: () {
    instant, e := IT.timestamp_from_millis(-1);

    assert(e.kind == .NONE);
    apollo := IT.timestamp_to_apollo(instant);
    back, be := IT.timestamp_from_apollo(apollo);

    assert(be.kind == .NONE && IT.equal(back, instant));
    calendar, ce := IT.date_time_to_calendar(.{ .{ 2024, 2, 29 }, .{ 12, 30, 40, 123000000 } });

    assert(ce.kind == .NONE && calendar.month_starting_at_0 == 1 && calendar.day_of_month_starting_at_0 == 28);
    civil, cve := IT.date_time_from_calendar(calendar);

    assert(cve.kind == .NONE && civil.time.nanosecond == 123000000);
    _, precision := IT.date_time_to_calendar(.{ .{ 2024, 1, 1 }, .{ 0, 0, 0, 1 } });

    assert(precision.kind == .INVALID_VALUE);
    duration := IT.Signed_Duration.{ -1, -500 };
    a := IT.duration_to_apollo(duration);
    d, de := IT.duration_from_apollo(a);

    assert(de.kind == .NONE && IT.equal(d, duration));
} @test

Time_Record :: struct {
    day: IT.Date; @json(with=time_json)
    time_json_encode :: IT.time_json_encode;
    time_json_decode :: IT.time_json_decode;
}

interop_json :: () {
    {
        record, e := IJ.from_json(Time_Record, "{\"day\":\"2024-02-29\"}");

        assert(e.kind == .NONE);
        text := IJ.to_json(record);

        assert(text == "{\"day\":\"2024-02-29\"}");
        free(text);
        IJ.free_json(*record);
        _, bad := IJ.from_json(Time_Record, "{\"day\":\"2024-02-30\"}");

        assert(bad.kind == .INVALID_VALUE);
        IJ.free_json(*bad);
    }

} @test
