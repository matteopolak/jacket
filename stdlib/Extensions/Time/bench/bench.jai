#import "Basic";
#import "Extensions/Testing";
BT :: #import "Extensions/Time";

civil_add_until :: (b: *Bench) {
    for 0..b.iterations - 1 {
        start := black_box(BT.Date.{ 2024, 1, 31 });
        end, e := BT.add(start, black_box(BT.Span.{ months = 1, days = 2 }));
        diff, de := BT.until(start, end);

        black_box(end);
        black_box(diff);
        black_box(e);
        black_box(de);
    }
} @bench

time_rfc3339_parse :: (b: *Bench) {
    text := "2024-03-10T07:30:00.123456789Z";

    set_bytes(b, text.count);

    for 0..b.iterations - 1 {
        v, e := BT.parse(BT.Timestamp, black_box(text));
        black_box(v);
        black_box(e);
    }
} @bench

time_rfc3339_format :: (b: *Bench) {
    instant, _ := BT.parse(BT.Timestamp, "2024-03-10T07:30:00.123456789Z");

    set_bytes(b, 30);

    for 0..b.iterations - 1 {
        builder: String_Builder;
        e := BT.format(*builder, black_box(instant));

        black_box(builder_string_length(*builder));
        black_box(e);
        free_buffers(*builder);
    }
} @bench

time_dst_to_civil :: (b: *Bench) {
    zone, e := BT.time_zone_get("America/New_York");

    assert(e.kind == .NONE);
    defer BT.time_zone_free(*zone);
    instant, _ := BT.timestamp_from_seconds(1710055800);

    reset_timer(b);

    for 0..b.iterations - 1 {
        v := BT.to_civil(black_box(zone), black_box(instant));
        black_box(v);
    }
} @bench

time_cached_zone :: (b: *Bench) {
    zone, e := BT.time_zone_get("America/New_York");

    assert(e.kind == .NONE);
    BT.time_zone_free(*zone);
    reset_timer(b);

    for 0..b.iterations - 1 {
        v, ve := BT.time_zone_get(black_box("America/New_York"));
        black_box(v);
        black_box(ve);
        BT.time_zone_free(*v);
    }
} @bench
