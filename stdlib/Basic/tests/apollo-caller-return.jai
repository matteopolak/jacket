// Reviewed basic.md: no-result #expand returns from a typed Apollo caller.
#import "Basic";

cleanups: s64;
evaluations: s64;

factor_value :: () -> s64 {
    evaluations += 1;
    return 1000000000000000;
}

scaled :: (m: s64, factor: s64) -> Apollo_Time {
    defer cleanups += 1;
    ConvertToApollo(m, factor);
}

seconds :: (m: s64) -> Apollo_Time {
    defer cleanups += 1;
    ConvertToApollo(m, factor_value());
}

main :: () {
    value := seconds(-17);
    expected := seconds_to_apollo(-17);

    assert(value.low == expected.low && value.high == expected.high);
    assert(cleanups == 1 && evaluations == 1);
    value = scaled(-1, -1);
    assert(value.low == 1 && value.high == 0);
    value = scaled(-1, 1);
    assert(value.low == 0xffffffffffffffff && value.high == -1);
    value = scaled(-0x8000000000000000, -0x8000000000000000);
    assert(value.low == 0 && value.high == 0x4000000000000000);
    value = scaled(0x7fffffffffffffff, 0x7fffffffffffffff);
    assert(value.low == 1 && value.high == 0x3fffffffffffffff);
    value = scaled(0, -0x8000000000000000);
    assert(value.low == 0 && value.high == 0);
    assert(cleanups == 6);
    print("ok\n");
}
