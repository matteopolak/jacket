#import "Basic";
// Unit-test the exact production arithmetic source, independently of Win32 ABI.
#load "../../stdlib/internal/time/filetime.jai";

main :: () {
    epoch := from_ticks(116444736000000000);

    assert(compare_apollo_times(epoch, APOLLO_TIME_FOR_JAN_1_1970) == 0);

    for ticks: u64.[0, 1, 0xffffffff, 0x100000000, 116444736000000000, 0x7fffffffffffffff, 0xffffffffffffffff] {
        at := from_ticks(ticks);
        actual, valid := to_ticks(at);
        assert(valid && actual == ticks);
    }

    zero := from_ticks(0);

    #if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
        calendar := to_calendar(zero);
        assert(calendar.year == 1601 && calendar.month_starting_at_0 == 0 && calendar.day_of_month_starting_at_0 == 0);
        assert(calendar.day_of_week_starting_at_0 == 1 && calendar.hour == 0 && calendar.minute == 0 && calendar.second == 0);
    }

    before := zero;
    before.low -= 1;
    value, valid := to_ticks(before);

    assert(!valid);
    value, valid = to_ticks(APOLLO_TIME_INVALID);
    assert(!valid);
    sub_tick := zero;
    sub_tick.low += 99999999;
    value, valid = to_ticks(sub_tick);
    assert(valid && value == 0);
    sub_tick.low += 1;
    value, valid = to_ticks(sub_tick);
    assert(valid && value == 1);
    value, valid = to_ticks(Apollo_Time.{ 0xffffffffffffffff, 0x7fffffffffffffff });
    assert(!valid);
    print("ok\n");
}

#scope_file
from_ticks :: (ticks: u64) -> Apollo_Time {
    low, high := ticks_to_words(ticks);
    return .{ low, high };
}

to_ticks :: (at: Apollo_Time) -> u64, bool {
    return words_to_ticks(at.low, at.high);
}
