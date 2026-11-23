// test: only(.MACOS)
// CFAbsoluteTime zero is 2001-01-01 00:00:00 UTC, a Monday, the first day of its year.
#import "Basic";
M :: #import "macOS";

main :: () {
    utc := M.CFTimeZoneCreateWithTimeIntervalFromGMT(null, 0);

    assert(utc != null);

    date := M.CFAbsoluteTimeGetGregorianDate(0, utc);

    print("date: %-%-%\n", date.year, date.month, date.day);
    print("time: %:%:%\n", date.hour, date.minute, cast(s64) date.second);
    weekday := M.CFAbsoluteTimeGetDayOfWeek(0, utc);

    print("week repeats after 7 days: %\n", M.CFAbsoluteTimeGetDayOfWeek(7 * 86400, utc) == weekday);
    print("next day differs: %\n", M.CFAbsoluteTimeGetDayOfWeek(86400, utc) != weekday);
    print("day of year: %\n", M.CFAbsoluteTimeGetDayOfYear(0, utc));
    print("now after epoch: %\n", M.CFAbsoluteTimeGetCurrent() > 0);
    M.CFRelease(utc);
}
