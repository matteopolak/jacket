#import "Basic";
#import "IntroSort";
#import "Random";

Item :: struct { index: int; }

main :: () {
    values: [..] int;
    random_seed(7);
    for 0..999 array_add(*values, cast(int) (random_get() % 500));
    intro_sort(values, (a: int, b: int) -> int { return a - b; });
    for 1..values.count-1 assert(values[it - 1] <= values[it]);

    items: [..] Item;
    for < 0..40 array_add(*items, .{it});
    intro_sort(items, (a: $T, b: T) -> int { return ifx a.index < b.index then -1 else 1; });
    for items assert(it.index == it_index);

    small := int.[3, 1, 2];
    insertion_sort(small, (a: int, b: int) -> int { return a - b; });
    assert(small[0] == 1 && small[2] == 3);
    print("ok\n");
}
