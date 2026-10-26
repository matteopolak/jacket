// Sorting orders any list, whatever its length, repeats or extremes.
#import "Basic";
#import "Extensions/Testing";

sorted_by_insertion :: (items: [] s64) -> [] s64 {
    result := NewArray(items.count, s64,, temp);

    for items {
        position := it_index;
        while position > 0 && result[position - 1] > it {
            result[position] = result[position - 1];
            position -= 1;
        }

        result[position] = it;
    }

    return result;
}

sorting_matches_a_simple_insertion_sort :: (items: [] s64) {
    want := sorted_by_insertion(items);
    got := Sort.quick_sort(items, (a, b) => compare_numbers(a, b));
    expect_equal(got, want);
} @test @property(300)

sorting_twice_changes_nothing :: (items: [] s64) {
    once := Sort.quick_sort(items, (a, b) => compare_numbers(a, b));
    copy := NewArray(once.count, s64,, temp);

    for once copy[it_index] = it;

    twice := Sort.quick_sort(once, (a, b) => compare_numbers(a, b));

    expect_equal(twice, copy);
} @test @property

sorting_by_key_orders_by_the_key :: (keys: [] s16) {
    Sort.quick_sort(keys, key => key);

    for 1..keys.count - 1 expect(keys[it - 1] <= keys[it]);
} @test @property

sorting_strings_orders_them_byte_by_byte :: (words: [] string) {
    Sort.quick_sort(words, Sort.compare_strings);

    for 1..words.count - 1 expect(Sort.compare_strings(words[it - 1], words[it]) <= 0);
} @test @property(200)

#scope_file

Sort :: #import "Sort";

compare_numbers :: (a: s64, b: s64) -> s64 {
    if a < b return -1;
    if a > b return 1;
    return 0;
}
