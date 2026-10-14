#import "Basic";

main :: () {
    times: [5] s64 = .[-9223372036854775808, -1001, 0, 1001, 9223372036854775807];
    for time: times {
        apollo := seconds_to_apollo(time);
        recovered, ok := to_seconds(apollo);

        assert(ok && recovered == time);
        millis := milliseconds_to_apollo(time);

        recovered, ok = to_milliseconds(millis);
        assert(ok && recovered == time);
    }

    rounded, ok := to_seconds(milliseconds_to_apollo(1500), true);

    assert(ok && rounded == 2);
    rounded, ok = to_seconds(milliseconds_to_apollo(-1500), true);
    assert(ok && rounded == -2);
    print("ok\n");
}
