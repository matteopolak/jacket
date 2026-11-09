// Conversions to Basic time values and JSON.

// Converts the Apollo epoch, rejecting subnanosecond precision instead of discarding it.
//
// ```jai
// instant, error := timestamp_from_apollo(APOLLO_TIME_FOR_JAN_1_1970);
// assert(error.kind == .NONE && instant.seconds == 0);
// ```
timestamp_from_apollo :: (value: Basic.Apollo_Time) -> Timestamp, Time_Error {
    delta := cast(S128) value - cast(S128) Basic.APOLLO_TIME_FOR_JAN_1_1970;
    q, r := wide_div(delta, 1000000);
    if r != 0 return .{}, .{ .INVALID_VALUE, "subnanosecond Apollo precision", -1 };
    return timestamp_wide(q);
}

// Converts a validated instant to Basic's femtosecond Apollo representation.
//
// ```jai
// instant, _ := timestamp_from_seconds(60);
// apollo := timestamp_to_apollo(instant);
// back, error := timestamp_from_apollo(apollo);
// assert(error.kind == .NONE && equal(back, instant));
// ```
timestamp_to_apollo :: (value: Timestamp) -> Basic.Apollo_Time {
    return Basic.seconds_to_apollo(value.seconds) + Basic.nanoseconds_to_apollo(value.nanoseconds) + Basic.APOLLO_TIME_FOR_JAN_1_1970;
}

// Treats Apollo as a duration (no epoch); subnanosecond input fails.
//
// ```jai
// span, _ := signed_duration(5);
// apollo := duration_to_apollo(span);
// back, error := duration_from_apollo(apollo);
// assert(error.kind == .NONE && back.seconds == 5);
// ```
duration_from_apollo :: (value: Basic.Apollo_Time) -> Signed_Duration, Time_Error {
    q, r := wide_div(cast(S128) value, 1000000);
    if r != 0 return .{}, .{ .INVALID_VALUE, "subnanosecond Apollo precision", -1 };
    return duration_wide(q);
}

// Converts an exact duration without adding an epoch.
//
// ```jai
// span, _ := signed_duration(5);
// apollo := duration_to_apollo(span);
// back, _ := duration_from_apollo(apollo);
// assert(equal(back, span));
// ```
duration_to_apollo :: (value: Signed_Duration) -> Basic.Apollo_Time {
    return Basic.seconds_to_apollo(value.seconds) + Basic.nanoseconds_to_apollo(value.nanoseconds);
}

// Validates Basic's zero-based components; ignores zone and derived weekday metadata.
//
// ```jai
// calendar, _ := date_time_to_calendar(.{ .{ 2024, 2, 29 }, .{ 12, 30, 0, 0 } });
// wall, error := date_time_from_calendar(calendar);
// assert(error.kind == .NONE && wall.date.day == 29 && wall.time.hour == 12);
// ```
date_time_from_calendar :: (value: Basic.Calendar_Time) -> Date_Time, Time_Error {
    return date_time(value.year, cast(s64) value.month_starting_at_0 + 1, cast(s64) value.day_of_month_starting_at_0 + 1, value.hour, value.minute, value.second, cast(s64) value.millisecond * 1000000);
}

// Converts a civil reading as UTC-tagged fields; rejects submillisecond precision.
//
// ```jai
// calendar, error := date_time_to_calendar(.{ .{ 2024, 2, 29 }, .{ 12, 30, 0, 0 } });
// assert(error.kind == .NONE && calendar.month_starting_at_0 == 1 && calendar.day_of_month_starting_at_0 == 28);
// ```
date_time_to_calendar :: (value: Date_Time) -> Basic.Calendar_Time, Time_Error {
    if value.time.nanosecond % 1000000 != 0 return .{}, .{ .INVALID_VALUE, "submillisecond calendar precision", -1 };
    return .{ year = value.date.year, month_starting_at_0 = value.date.month - 1, day_of_month_starting_at_0 = value.date.day - 1, day_of_week_starting_at_0 = cast(s8) (cast(s64) weekday(value.date) % 7), hour = value.time.hour, minute = value.time.minute, second = value.time.second, millisecond = cast(s16) (value.time.nanosecond / 1000000), time_zone = .UTC }, .{};
}

// Uses [Json_Writer](https://github.com/matteopolak/jacket/blob/main/docs/extensions/json.md).
// Custom Json field encoder; canonical ISO strings use Json's writer limits.
//
// ```jai,skip_on(wasm)
// #import "Extensions/Json";
// T :: #import "Extensions/Time";
//
// Event :: struct {
//     day: Date; @json(with=time_json)
//     time_json_encode :: T.time_json_encode;
//     time_json_decode :: T.time_json_decode;
// }
//
// main :: () {
//     event, error := from_json(Event, "{\"day\":\"2024-02-29\"}");
//     assert(error.kind == .NONE && event.day.day == 29);
//     text := to_json(event);
//     defer free(text);
//     assert(text == "{\"day\":\"2024-02-29\"}");
// }
// ```
time_json_encode :: (writer: *Json_Writer, value: *$T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned, Span]) {
    b: String_Builder;

    defer free_buffers(*b);
    e := format(*b, value.*);
    if e.kind != .NONE {
        writer.builder.failed = true;
        return;
    }

    text := builder_to_string(*b);

    defer free(text);
    Json.json_write_string(writer, text);
}

// Uses [Json_Reader](https://github.com/matteopolak/jacket/blob/main/docs/extensions/json.md).
// Custom Json field decoder; parse errors become Json INVALID_VALUE and output stays unchanged.
//
// ```jai,skip_on(wasm)
// #import "Extensions/Json";
// T :: #import "Extensions/Time";
//
// Event :: struct {
//     day: Date; @json(with=time_json)
//     time_json_encode :: T.time_json_encode;
//     time_json_decode :: T.time_json_decode;
// }
//
// main :: () {
//     _, error := from_json(Event, "{\"day\":\"2024-02-30\"}");
//     assert(error.kind == .INVALID_VALUE);
// }
// ```
time_json_decode :: (reader: *Json_Reader, out: *$T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned, Span]) -> bool {
    text, ok := Json.json_read_name(reader);
    if !ok return false;

    v, e := parse(T, text);
    if e.kind != .NONE return Json.json_fail(reader, .INVALID_VALUE, e.message);

    // Json has no cleanup hook for owning with= handlers when a later field fails.
    #if T == Time_Zone {
        if v.kind == .RULES {
            time_zone_free(*v);
            return Json.json_fail(reader, .INVALID_VALUE, "rule-zone Json decoding needs an owning handler cleanup hook");
        }
    }

    #if T == Zoned {
        if v.zone.kind == .RULES {
            time_zone_free(*v.zone);
            return Json.json_fail(reader, .INVALID_VALUE, "rule-zone Json decoding needs an owning handler cleanup hook");
        }
    }

    out.* = v;

    return true;
}

#scope_module

Builder :: Basic.String_Builder;

Json_Writer :: Json.Json_Writer;

Json_Reader :: Json.Json_Reader;
