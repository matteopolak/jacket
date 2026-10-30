#import "Basic";
S :: #import "Sort";
#import "Math";

main :: () {
    // NaN is ordered after every number, so sorting floats that hold NaN is consistent.
    nan := FLOAT32_NAN;

    assert(S.compare_floats(nan, nan) == 0);
    assert(S.compare_floats(nan, 1) == 1 && S.compare_floats(1, nan) == -1);
    assert(S.compare_floats(nan, FLOAT32_INFINITY) == 1 && S.compare_floats(-FLOAT32_INFINITY, nan) == -1);
    assert(S.compare_floats(0, -0.0) == 0);

    floats := NewArray(300, float,, temp);
    for *floats it.* = ifx it_index % 5 == 0 then nan else cast(float) ((it_index * 37) % 101) - 50;

    S.quick_sort(floats, S.compare_floats);

    seen_nan: bool;

    for floats {
        if it != it {
            seen_nan = true;
        } else {
            assert(!seen_nan, "a number sorted after a NaN");
            if it_index > 0 && floats[it_index - 1] == floats[it_index - 1] assert(floats[it_index - 1] <= it);
        }
    }

    assert(seen_nan);
    print("ok\n");
}
