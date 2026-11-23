// intro_sort orders integers ascending for empty, tiny, repeated, presorted, reversed and random inputs.
#import "Basic";
Sort :: #import "IntroSort";
R :: #import "Random";

ascending :: (a: int, b: int) -> int {
    return a - b;
}

descending :: (a: int, b: int) -> int {
    return b - a;
}

// Checks each element against the one before it.
check_ascending :: (values: [..] int) {
    for 1..values.count - 1 assert(values[it - 1] <= values[it]);
}

// Checks that `sorted` is exactly `expected`, element by element.
check_equal :: (sorted: [..] int, expected: [..] int) {
    assert(sorted.count == expected.count);
    for i: 0..sorted.count - 1 assert(sorted[i] == expected[i]);
}

// Insertion sort written out here, as an independent reference for the random input.
reference_sort :: (values: [..] int) -> [..] int {
    result: [..] int;

    for values {
        array_add(*result, it);
        j := result.count - 1;
        while j > 0 && result[j - 1] > result[j] {
            tmp := result[j];
            result[j] = result[j - 1];
            result[j - 1] = tmp;
            j -= 1;
        }
    }

    return result;
}

main :: () {
    empty: [..] int;

    Sort.intro_sort(empty, ascending);
    assert(empty.count == 0);

    one := int.[7];

    Sort.intro_sort(one, ascending);
    assert(one[0] == 7);

    pair := int.[9, 3];

    Sort.intro_sort(pair, ascending);
    assert(pair[0] == 3 && pair[1] == 9);

    same: [..] int;
    for 0..299 array_add(*same, 5);

    Sort.intro_sort(same, ascending);
    for same assert(it == 5);

    presorted: [..] int;
    for 0..999 array_add(*presorted, it);

    Sort.intro_sort(presorted, ascending);
    for presorted assert(it == it_index);

    reversed: [..] int;
    for 0..999 array_add(*reversed, 999 - it);

    Sort.intro_sort(reversed, ascending);
    for reversed assert(it == it_index);

    // Rises from 0 to 499, then falls back from 498 to 0: 999 values, minimum 0, maximum 499.
    organ: [..] int;
    for 0..499 array_add(*organ, it);
    for 0..498 array_add(*organ, 498 - it);

    Sort.intro_sort(organ, ascending);
    check_ascending(organ);
    assert(organ.count == 999);
    assert(organ[0] == 0 && organ[organ.count - 1] == 499);

    back: [..] int;
    for 0..99 array_add(*back, it);

    Sort.intro_sort(back, descending);
    for back assert(it == 99 - it_index);

    R.random_seed(cast(u64) 77);
    random_values: [..] int;
    for 0..499 array_add(*random_values, (cast(int) (R.random_get() % 1000)) - 500);

    expected := reference_sort(random_values);

    Sort.intro_sort(random_values, ascending);
    check_equal(random_values, expected);

    print("ok\n");
}
