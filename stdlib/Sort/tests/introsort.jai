// quick_sort and IntroSort's intro_sort sort every input shape, and the key overload computes each
// key once and keeps equal keys in order.
#import "Basic";
S :: #import "Sort";
I :: #import "IntroSort";

key_calls: s64;

Item :: struct {
    key: s64;
    identity: s64;
}

key_of :: (item: Item) -> s64 {
    key_calls += 1;
    return item.key;
}

ascending :: (a: s64, b: s64) -> s64 {
    if a < b return -1;
    if a > b return 1;
    return 0;
}

fill :: (values: [] s64, shape: s64) {
    seed: u64 = 12345;

    for *value, index: values {
        seed = seed * 6364136223846793005 + 1442695040888963407;

        if shape == 0 value.* = index;
        else if shape == 1 value.* = values.count - index;
        else if shape == 2 value.* = 7;
        else if shape == 3 value.* = ifx index < values.count / 2 then index else values.count - index;
        else if shape == 4 value.* = cast(s64) (seed >> 40) % 1000;
        else value.* = index % 3;
    }
}

check_sorted :: (values: [] s64) {
    for index: 1..values.count - 1 assert(values[index - 1] <= values[index]);
}

checksum :: (values: [] s64) -> s64 {
    total: s64;
    for value: values total += value * value + value;
    return total;
}

main :: () {
    for length: s64.[0, 1, 2, 3, 15, 16, 17, 31, 100, 1000, 5000] {
        for shape: 0..5 {
            storage := NewArray(length, s64);

            defer array_free(storage);

            fill(storage, shape);
            expected := checksum(storage);
            copy := array_copy(storage);

            defer array_free(copy);

            I.intro_sort(storage, ascending);
            check_sorted(storage);
            assert(checksum(storage) == expected);

            result := S.quick_sort(copy, ascending);

            assert(result.data == copy.data);
            check_sorted(copy);
            assert(checksum(copy) == expected);
        }
    }

    items: [200] Item;
    for *item, index: items item.* = .{ (index * 7) % 5, index };

    key_calls = 0;
    S.quick_sort(items, key_of);
    assert(key_calls == items.count);

    for index: 1..items.count - 1 {
        assert(items[index - 1].key <= items[index].key);
        if items[index - 1].key == items[index].key assert(items[index - 1].identity < items[index].identity);
    }

    empty: [] Item;

    key_calls = 0;
    S.quick_sort(empty, key_of);
    assert(key_calls == 0);

    one: [1] Item = .[.{ 4, 0 }];

    S.quick_sort(one, key_of);
    assert(one[0].key == 4 && key_calls == 0);
    print("ok\n");
}
