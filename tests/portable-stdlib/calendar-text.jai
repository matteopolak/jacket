// Reviewed basic.md calendar text, temporary lifetime and expansion value.
#import "Basic";

main :: () {
    mark := get_temporary_storage_mark();

    defer set_temporary_storage_mark(mark);
    ct := Calendar_Time.{ year = 2000, month_starting_at_0 = 1, day_of_month_starting_at_0 = 28,
                         hour = 3, minute = 4, second = 5, millisecond = 6, time_zone = .UTC };
    english := calendar_to_string(ct);
    iso := calendar_to_iso_string(ct);

    assert(english == "29 February 2000, 03:04:05");
    assert(iso == "2000-02-29T03:04:05.006Z");
    ct.time_zone = .LOCAL;
    assert(calendar_to_iso_string(ct) == "2000-02-29T03:04:05.006");
    ct.month_starting_at_0 = -128;
    assert(calendar_to_string(ct) == "29 January 2000, 03:04:05");
    ct.month_starting_at_0 = 127;
    assert(calendar_to_string(ct) == "29 December 2000, 03:04:05");
    assert(english == "29 February 2000, 03:04:05" && iso == "2000-02-29T03:04:05.006Z");

    expected_months := string.["29 January 2000, 03:04:05", "29 February 2000, 03:04:05",
                              "29 March 2000, 03:04:05", "29 April 2000, 03:04:05",
                              "29 May 2000, 03:04:05", "29 June 2000, 03:04:05",
                              "29 July 2000, 03:04:05", "29 August 2000, 03:04:05",
                              "29 September 2000, 03:04:05", "29 October 2000, 03:04:05",
                              "29 November 2000, 03:04:05", "29 December 2000, 03:04:05"];

    for month: 0..11 {
        ct.month_starting_at_0 = cast(s8) month;
        text := calendar_to_string(ct);
        assert(text == expected_months[month]);
    }

    print("ok\n");
}
