// Sort's entry points: comparator and key quick_sort (returning the array), bubble_sort,
// and lambdas as comparators.
#import "Basic";
#import "Sort";
#import "Random";

Region :: struct { start: s32; }

sorted :: (a: [] $T, f: (T, T) -> $R) -> bool {
    for 1..a.count-1  if f(a[it-1], a[it]) > 0  return false;
    return true;
}

main :: () {
    values: [..] int;
    for 1..500  array_add(*values, cast(int) (random_get() % 1000));
    result := quick_sort(values, (a, b) => a - b);
    assert(result.data == values.data && sorted(values, (a: int, b: int) -> int { return a - b; }));

    quick_sort(values, x => -x);
    assert(values[0] >= values[values.count - 1]);

    regions: [..] Region;
    for 0..40  array_add(*regions, .{ cast(s32) ((it * 37) % 41) });
    quick_sort(regions, (a, b) => (a.start - b.start));
    for 0..40  assert(regions[it].start == it);

    names := string.["pear", "apple", "fig", "app"];
    quick_sort(names, compare_strings);
    assert(names[0] == "app" && names[1] == "apple" && names[3] == "pear");

    floats := float.[3, 1, 2];
    bubble_sort(floats, compare_floats);
    assert(floats[0] == 1 && floats[2] == 3);
}
