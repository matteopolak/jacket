// A by-value `it` over an array of aggregates is the element itself: `*it` points into the array.
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

main :: () {
    entries: [..] Entry;

    array_add(*entries, .{ key = "a" });
    array_add(*entries, .{ key = "b" });

    array_add(*find(entries, "b").values, 1);
    array_add(*find(entries, "b").values, 2);
    assert(entries[1].values.count == 2);

    pointers: [2] *Entry;
    for entries pointers[it_index] = *it;
    for pointers assert(it == *entries[it_index]);

    print("ok\n");
}
