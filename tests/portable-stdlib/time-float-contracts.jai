#import "Basic";
B :: #import "Basic";

main :: () {
    #assert !#exists(B.seconds_with_fraction);
    positive: Apollo_Time = .{ 1, 0 };
    negative: Apollo_Time = .{ 0xffffffffffffffff, -1 };
    plus, plus_ok := to_float64_seconds(positive);
    minus, minus_ok := to_float64_seconds(negative);

    if !plus_ok || !minus_ok || plus <= 0 || minus >= 0 exit(2);
    // Sign and approximate magnitude; no general ULP/rounding guarantee.
    if minus != -plus || plus < 1e-16 || plus > 1e-14 exit(3);

    for value: s64.[-1000001, -1000, -1, 0, 1, 1000, 1000001] {
        actual, valid := to_float64_seconds(nanoseconds_to_apollo(value));
        if !valid exit(4);
        if value < 0 && actual >= 0 exit(5);
        if value > 0 && actual <= 0 exit(6);

        expected := cast(float64) value / 1000000000.0;
        error := actual - expected;

        if error < 0 error = -error;
        if error > 1e-12 exit(7);
    }

    largest := seconds_to_apollo(0x7fffffffffffffff);
    result, valid := to_float64_seconds(largest);

    if !valid || result <= 0 exit(8);

    smallest := seconds_to_apollo(-0x8000000000000000);

    result, valid = to_float64_seconds(smallest);
    if !valid || result >= 0 exit(9);

    outside := to_apollo(9223372036854775808.0);

    result, valid = to_float64_seconds(outside);
    if valid || result != 0 exit(10);

    outside = to_apollo(-9223372036854777856.0);
    result, valid = to_float64_seconds(outside);
    if valid || result != 0 exit(11);

    result, valid = to_float64_seconds(APOLLO_TIME_INVALID);
    if valid || result != 0 exit(12);

    write_string("ok\n");
}
