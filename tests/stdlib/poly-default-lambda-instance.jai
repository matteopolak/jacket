// A polymorphic procedure whose `$compare` default is a lambda is instantiated once per type,
// not once per call: the default is the same procedure on every call.
#import "Basic";

compare_of :: (x: $T, $compare: (T, T) -> bool = (a, b) => a == b) -> u64 {
    return cast(u64) compare;
}

count_equal :: (arr: []$T, item: T, $compare: (T, T) -> bool = (a, b) => a == b) -> s64 {
    n := 0;
    for arr  if compare(it, item) n += 1;
    return n;
}

main :: () {
    a := compare_of(1);
    b := compare_of(2);
    c := compare_of(1.0);
    assert(a == b, "same T, same default lambda");
    assert(a != c, "different T, different instance");
    arr := s32.[1, 2, 1];
    assert(count_equal(arr, 1) == 2);
    assert(count_equal(arr, 2) == 1);
    print("ok\n");
}
