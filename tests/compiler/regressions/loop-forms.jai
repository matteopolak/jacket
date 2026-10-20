// Loop forms: ranges, named iterators, reverse, nesting with labels, while, and `remove`.
#import "Basic";

collect :: (values: [] int) -> string {
    b: String_Builder;

    for values {
        if it_index > 0 append(*b, ",");
        print_to_builder(*b, "%", it);
    }

    return builder_to_string(*b);
}

main :: () {
    // Ranges are inclusive and `it_index` counts iterations.
    sum := 0;
    for 1..5 sum += it;

    assert(sum == 15);

    last_index := -1;
    for 3..7 last_index = it_index;

    assert(last_index == 4);

    // A named iterator, with the index still named it_index.
    total := 0;

    for i: 1..4 {
        total += i * it_index;
    }

    assert(total == 1 * 0 + 2 * 1 + 3 * 2 + 4 * 3);

    // A named index.
    for v, i: 10..12 {
        assert(v == 10 + i);
    }

    // Reverse iteration still writes the range as low..high.
    seen: [..] int;
    for < 1..4 array_add(*seen, it);

    assert(collect(seen) == "4,3,2,1");
    array_free(seen);

    // An empty range runs zero times.
    ran := false;
    for 5..4 ran = true; // jailint: allow(reversed_range) the empty range is what is tested

    assert(!ran);

    // Arrays, views and strings.
    fixed := int.[4, 5, 6];

    assert(collect(fixed) == "4,5,6");
    view: [] int = fixed;
    count := 0;

    for view count += 1;

    assert(count == 3);

    text := "abc";
    codes := 0;

    for text codes += it;

    assert(codes == 97 + 98 + 99);

    // Iteration by pointer modifies the collection.
    values := int.[1, 2, 3];
    for *values it.* *= 10;

    assert(collect(values) == "10,20,30");

    for *p: values p.* += 1;

    assert(collect(values) == "11,21,31");

    // Reverse by pointer, with a named index.
    order: [..] int;
    for < *p, i: values array_add(*order, p.* + i);

    assert(collect(order) == "33,22,11");
    array_free(order);

    // break and continue act on the innermost loop.
    hits := 0;

    for 0..3 {
        for 0..3 {
            if it == 2 break;
            hits += 1;
        }
    }

    assert(hits == 8);

    // Labels pick an outer loop.
    pairs := 0;

    for outer: 0..4 {
        for j: 0..4 {
            if j == 2 continue outer;
            if outer == 3 break outer;
            pairs += 1;
        }
    }

    assert(pairs == 6);

    // A loop is named by its iterator.
    found := -1;

    for search: 0..9 {
        for j: 0..9 {
            if search * j == 42 {
                found = search * 10 + j;
                break search;
            }
        }
    }

    assert(found == 67);

    // while checks its condition before each iteration.
    n := 0;
    while n < 5 n += 1;

    assert(n == 5);

    skipped := true;
    while false skipped = false;

    assert(skipped);

    even_sum := 0;
    k := 0;

    while k < 10 {
        k += 1;
        if k % 2 continue;
        even_sum += k;
    }

    assert(even_sum == 2 + 4 + 6 + 8 + 10);

    stop := 0;

    while true {
        stop += 1;
        if stop == 7 break;
    }

    assert(stop == 7);

    // `remove` swaps the last element in and revisits the slot.
    items: [..] int;
    for 1..8 array_add(*items, it);

    for items {
        if it % 2 == 0 remove it;
    }

    assert(items.count == 4);
    odd := 0;

    for items {
        assert(it % 2 == 1);
        odd += it;
    }

    assert(odd == 1 + 3 + 5 + 7);

    // The same through a pointer, and by name.
    for *p: items {
        if p.* == 3 remove p;
    }

    assert(items.count == 3);

    for v: items {
        if v == 5 remove v;
    }

    assert(items.count == 2);
    array_free(items);

    // Removing every element leaves an empty array.
    everything: [..] int;
    for 1..5 array_add(*everything, it);
    for everything remove it;

    assert(everything.count == 0);
    array_free(everything);

    print("ok\n");
}
