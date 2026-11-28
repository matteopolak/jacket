// Exact instants, civil values, explicit zones and calendar-aware spans.
//
// Types never mix by accident: a [Timestamp] is an exact instant, a [Date_Time] is a wall-clock
// reading with no zone, and a [Zoned_Date_Time] is an instant read in an explicit [Time_Zone]. Every
// fallible procedure returns a [Time_Error]; `error.kind == .NONE` means success.
//
// A zone from [time_zone_get], [time_zone_system], [time_zone_from_tzif] or
// [time_zone_from_posix] holds shared rules: release it with [time_zone_free], and keep a copy
// past that with [time_zone_retain]. UTC and fixed-offset zones allocate nothing. A
// [Zoned_Date_Time] borrows its zone. Parsers check their input against [Time_Options] before
// allocating.
//
// Civil values and calendar arithmetic:
//
// ```jai
// day, error := date(2024, 1, 31);
// assert(error.kind == .NONE);
// next_month, _ := add(day, Span.{ months = 1 });
// assert(next_month.month == 2 && next_month.day == 29); // clamped to the month's end
// assert(weekday(next_month) == .THURSDAY);
// ```
//
// Instants, zones and daylight-saving gaps:
//
// ```jai
// zone, error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(error.kind == .NONE);
// defer time_zone_free(*zone);
//
// // 02:30 does not exist on 2024-03-10; the default policy moves it forward.
// wall := Date_Time.{ .{ 2024, 3, 10 }, .{ 2, 30, 0, 0 } };
// z, _ := to_zoned_date_time(wall, zone);
// assert(z.offset.seconds == -14400);
// _, rejected := to_zoned_date_time(wall, zone, .REJECT);
// assert(rejected.kind == .GAP);
// ```
//
// Text:
//
// ```jai
// instant, _ := parse(Timestamp, "2024-03-10T07:30:00Z");
// builder: String_Builder;
// defer free_buffers(*builder);
// format(*builder, instant, "%d %b %Y");
// text := builder_to_string(*builder);
// defer free(text);
// assert(text == "10 Mar 2024");
// ```

#load "types.jai";
#load "numeric.jai";
#load "civil.jai";
#load "instants.jai";
#load "spans.jai";
#load "zone_load.jai";
#load "zoned_date_time.jai";
#load "ordering.jai";
#load "text.jai";
#load "pattern.jai";
#load "interop.jai";

#scope_module

Basic :: #import "Basic";
#import "String";
Json :: #import "Extensions/Json";

using,except(Time_Zone) Basic;
NS :: 1000000000;
DAY_NS :: 86400000000000;
MIN_I64 :: -9223372036854775808;
MAX_I64 :: 9223372036854775807;
