// A by-value `it` over an array is the element itself: `*it` points into the array.
// rules: flow.28
#import "Basic";

Entry :: struct {
    key: string;
    values: [..] int;
}

find :: (entries: [] Entry, key: string) -> *Entry {
    for entries if it.key == key return *it;
    return null;
}

reset :: (n: *int) {
    n.* = 0;
}

main :: () {
    entries: [..] Entry;
    array_add(*entries, .{ key = "a" });
    array_add(*entries, .{ key = "b" });

    array_add(*find(entries, "b").values, 1);
    array_add(*find(entries, "b").values, 2);
    assert(entries[1].values.count == 2);

    numbers := int.[1, 2, 3];
    for numbers reset(*it);
    assert(numbers[0] == 0 && numbers[1] == 0 && numbers[2] == 0);

    pointers: [3] *int;
    for numbers pointers[it_index] = *it;
    for pointers assert(it == *numbers[it_index]);
    print("ok\n");
}
