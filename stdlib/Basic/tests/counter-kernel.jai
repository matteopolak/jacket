#import "Basic";
#load "../../internal/time/counter.jai";

main :: () {
    for count: s64.[0, 1, 17, 9223372036854775807] {
        actual := from_counter(count, 1);
        expected := seconds_to_apollo(count);
        assert(compare_apollo_times(actual, expected) == 0);
    }

    third := from_counter(1, 3);

    assert(third.low == 333333333333333 && third.high == 0);
    fractional := from_counter(10000001, 10000000);

    assert(compare_apollo_times(fractional, nanoseconds_to_apollo(1000000100)) == 0);
    assert(compare_apollo_times(from_counter(9223372036854775807, 9223372036854775807), seconds_to_apollo(1)) == 0);
    assert(compare_apollo_times(from_counter(1, 9223372036854775807), seconds_to_apollo(0)) == 0);
    assert(compare_apollo_times(from_counter(-1, 1), APOLLO_TIME_INVALID) == 0);
    assert(compare_apollo_times(from_counter(1, 0), APOLLO_TIME_INVALID) == 0);
    assert(compare_apollo_times(from_counter(1, -1), APOLLO_TIME_INVALID) == 0);
    previous := from_counter(0, 1009);

    for count: 1..2048 {
        current := from_counter(count, 1009);
        assert(compare_apollo_times(current, previous) >= 0);
        previous = current;
    }

    print("ok\n");
}

#scope_file
from_counter :: (counter: s64, frequency: s64) -> Apollo_Time {
    low, high, valid := counter_to_words(counter, frequency);
    if !valid return APOLLO_TIME_INVALID;
    return .{ low, high };
}
