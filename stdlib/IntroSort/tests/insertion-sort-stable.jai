// insertion_sort orders by key and keeps equal keys in their original order; intro_sort orders by key too.
#import "Basic";
Sort :: #import "IntroSort";

Item :: struct {
    key: int;
    tag: int;
}

by_key :: (a: Item, b: Item) -> int {
    return a.key - b.key;
}

// Checks that keys do not decrease and that, among equal keys, tags keep their original order.
check_stable :: (items: [..] Item) {
    for 1..items.count - 1 {
        before := items[it - 1];
        after := items[it];
        assert(before.key <= after.key);
        if before.key == after.key assert(before.tag < after.tag);
    }
}

main :: () {
    empty: [..] Item;

    Sort.insertion_sort(empty, by_key);
    assert(empty.count == 0);

    items: [..] Item;
    for 0..299 array_add(*items, Item.{ key = (it * 7) % 5, tag = it });

    Sort.insertion_sort(items, by_key);
    assert(items.count == 300);
    check_stable(items);
    assert(items[0].key == 0 && items[items.count - 1].key == 4);

    // Nearly sorted input: every pair of neighbours is swapped once.
    nearly: [..] Item;
    for 0..199 array_add(*nearly, Item.{ key = it, tag = it });

    for i: 0..98 {
        tmp := nearly[2 * i];
        nearly[2 * i] = nearly[2 * i + 1];
        nearly[2 * i + 1] = tmp;
    }

    Sort.insertion_sort(nearly, by_key);
    check_stable(nearly);
    for nearly assert(it.key == it_index);

    // intro_sort is not promised to be stable, so only the key order is checked here.
    grouped: [..] Item;
    for 0..499 array_add(*grouped, Item.{ key = (it * 13) % 17, tag = it });

    Sort.intro_sort(grouped, by_key);
    for 1..grouped.count - 1 assert(grouped[it - 1].key <= grouped[it].key);

    print("ok\n");
}
