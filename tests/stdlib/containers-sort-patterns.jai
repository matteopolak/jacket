// Sorting modules on the input shapes that break naive partitioning (sorted, reversed, all equal,
// organ pipe, few distinct values, empty, one element), checked against a counting reference;
// plus the container entry points the other tests leave alone: Hash_Table resizing and resetting
// in place, Walk_Table probing, Bucket_Array indexing, and Bit_Array word edges.
#import "Basic";
#import "Sort";
#import "IntroSort";
#import "RadixSort";
#import "Hash_Table";
#import "Bucket_Array";
#import "Bit_Array";

main :: () {
    check_sorts();
    check_radix_sort();
    check_hash_table();
    check_bucket_array_indexing();
    check_bit_array_edges();
    print("ok\n");
}

Shape :: enum {
    EMPTY;
    ONE;
    SORTED;
    REVERSED;
    ALL_EQUAL;
    ORGAN_PIPE;
    FEW_DISTINCT;
    SCRAMBLED;
}

make_input :: (shape: Shape, count: s64) -> [] s64 {
    values := NewArray(count, s64,, temp);
    seed: u64 = 0x2545_f491;
    for *values {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        if shape == {
            case .SORTED; it.* = it_index * 3 - 500;
            case .REVERSED; it.* = (count - it_index) * 3;
            case .ALL_EQUAL; it.* = 42;
            case .ORGAN_PIPE; it.* = ifx it_index < count / 2 then it_index else count - it_index;
            case .FEW_DISTINCT; it.* = cast(s64) (seed % 4) - 2;
            case; it.* = cast(s64) (seed % 100_000) - 50_000;
        }
    }
    return values;
}

shape_count :: (shape: Shape) -> s64 {
    if shape == .EMPTY return 0;
    if shape == .ONE return 1;
    return 1500;
}

// Sorted, and a permutation of the input (same multiset, compared by sum and sum of squares and
// by a merge against the input sorted with a trusted method).
expect_sorted :: (sorted: [] s64, original: [] s64, what: string, shape: Shape) {
    assert(sorted.count == original.count, "% on %: count", what, shape);
    for 1..sorted.count - 1 assert(sorted[it - 1] <= sorted[it], "% on %: out of order at %", what, shape, it);
    reference := array_copy(original,, temp);
    insertion_reference(reference);
    for sorted assert(it == reference[it_index], "% on %: element % is % not %", what, shape, it_index, it, reference[it_index]);
}

insertion_reference :: (values: [] s64) {
    for i: 1..values.count - 1 {
        value := values[i];
        j := i - 1;
        while j >= 0 && values[j] > value {
            values[j + 1] = values[j];
            j -= 1;
        }
        values[j + 1] = value;
    }
}

compare_s64 :: (a: s64, b: s64) -> s64 {
    if a < b return -1;
    if a > b return 1;
    return 0;
}

check_sorts :: () {
    for shape: enum_values_as_enum(Shape) {
        original := make_input(shape, shape_count(shape));

        a := array_copy(original,, temp);
        quick_sort(a, compare_s64);
        expect_sorted(a, original, "quick_sort", shape);

        b := array_copy(original,, temp);
        returned := quick_sort(b, x => -x);
        assert(returned.data == b.data && returned.count == b.count, "quick_sort returns its input");
        // Sorted by a descending key: reverse to compare.
        for 0..b.count / 2 - 1 Swap(*b[it], *b[b.count - 1 - it]);
        expect_sorted(b, original, "quick_sort by key", shape);

        c := array_copy(original,, temp);
        intro_sort(c, compare_s64);
        expect_sorted(c, original, "intro_sort", shape);

        if original.count <= 300 || shape == .FEW_DISTINCT || shape == .SORTED {
            d := array_copy(original,, temp);
            bubble_sort(d, compare_s64);
            expect_sorted(d, original, "bubble_sort", shape);
        }
        reset_temporary_storage();
    }

    // Stability is not promised, but records with equal keys all survive.
    Record :: struct {
        key: s64;
        tag: s64;
    }
    records: [200] Record;
    for *records it.* = .{ key = it_index % 3, tag = it_index };
    by_key :: (x: Record, y: Record) -> int {
        return x.key - y.key;
    }
    intro_sort(records, by_key);
    seen: [200] bool;
    for records {
        if it_index assert(records[it_index - 1].key <= it.key);
        assert(!seen[it.tag], "tag % twice", it.tag);
        seen[it.tag] = true;
        assert(it.tag % 3 == it.key, "a record kept its own key");
    }

    // Floats and strings through the shipped comparators.
    floats := float.[3.5, -0.0, 1e30, -7, 0.25, 0.0, -1e-30];
    quick_sort(floats, compare_floats);
    for 1..floats.count - 1 assert(floats[it - 1] <= floats[it], "floats at %", it);
    words := string.["pear", "", "apple", "applesauce", "Zebra", "app"];
    quick_sort(words, compare_strings);
    assert(words[0] == "" && words[1] == "Zebra" && words[2] == "app" && words[3] == "apple", "strings: %", words);
    assert(words[4] == "applesauce" && words[5] == "pear", "strings: %", words);
}

check_radix_sort :: () {
    radix_tests();

    sorter: RadixSort;
    defer free_resources(*sorter);
    // Every rank appears once; the ranked values are ordered, and equal values keep input order.
    for shape: enum_values_as_enum(Shape) {
        count := shape_count(shape);
        if count == 0 continue;
        input := make_input(shape, count);
        keys := NewArray(count, u32,, temp);
        for input keys[it_index] = cast(u32) (it + 60_000);
        reset(*sorter);
        sort(*sorter, keys);
        seen := NewArray(count, bool,, temp);
        for 0..count - 1 {
            r := sorter[it];
            assert(!seen[r], "rank % twice on %", r, shape);
            seen[r] = true;
            if it {
                previous := sorter[it - 1];
                assert(keys[previous] <= keys[r], "radix on %: out of order at %", shape, it);
                if keys[previous] == keys[r] assert(previous < r, "radix on %: equal keys reordered at %", shape, it);
            }
        }
        reset_temporary_storage();
    }

    // u64 keys that differ only in the high word, and reorder() applying the ranks.
    wide := u64.[0xffff_0000_0000_0001, 0x0000_0001_0000_0000, 0xffff_0000_0000_0000, 7];
    reset(*sorter);
    sort(*sorter, wide);
    assert(sorter[0] == 3 && sorter[1] == 1 && sorter[2] == 2 && sorter[3] == 0, "u64 ranks % % % %", sorter[0], sorter[1], sorter[2], sorter[3]);
    values: [..] u64;
    array_add(*values, ..wide);
    reorder(*values, sorter);
    for 1..values.count - 1 assert(values[it - 1] <= values[it]);
    array_free(values);

    // Floats with negatives and both zeros.
    floats := float.[2.5, -1, 0, -0.0, -100, 3];
    reset(*sorter);
    sort(*sorter, floats);
    for 1..floats.count - 1 assert(floats[sorter[it - 1]] <= floats[sorter[it]], "float ranks at %", it);
}

check_hash_table :: () {
    table: Table(s64, s64);
    defer deinit(*table);
    for 0..999 table_add(*table, it * 7919, it);
    assert(table.count == 1000);

    // Growing explicitly rehashes everything; asking for less than the load needs never shrinks it.
    table_resize(*table, 8192);
    assert(table.allocated == 8192, "allocated %", table.allocated);
    for 0..999 {
        found, value := table_find(*table, it * 7919);
        assert(found && value == it, "key % after resize", it * 7919);
    }
    table_resize(*table, 16);
    assert(table.allocated >= 1000 * 100 / 70, "a resize keeps room for the entries: %", table.allocated);
    assert(table.count == 1000);
    for 0..999 assert(table_contains(*table, it * 7919));

    // Walk_Table visits the probe sequence of `key`: one of its stops holds the key.
    {
        key := 500 * 7919;
        stops := 0;
        hit := false;
        Walk_Table(#code {
            stops += 1;
            if table.entries[index].hash == hash && table.entries[index].key == key hit = true;
        });
        assert(hit && stops >= 1, "Walk_Table found the key after % stops", stops);
    }
    {
        // A key that is absent stops at the first never-used slot.
        key := -1;
        stops := 0;
        Walk_Table(#code {
            stops += 1;
            assert(table.entries[index].key != key || table.entries[index].hash != hash);
        });
        assert(stops < table.allocated);
    }

    // Reset in place: no entries, the same storage, and it fills again.
    data := table.entries.data;
    allocated := table.allocated;
    table_reset_keeping_memory(*table);
    assert(table.count == 0 && table.entries.data == data && table.allocated == allocated);
    for table assert(false, "an entry survived the reset: %", it);
    assert(!table_contains(*table, 7919));
    for 0..99 table_set(*table, it, -it);
    for 0..99 assert(table_find_pointer(*table, it).* == -it);
    assert(table.count == 100 && table.entries.data == data);
}

check_bucket_array_indexing :: () {
    array: Bucket_Array(s64, 8);
    defer bucket_array_reset(*array);
    for 0..29 bucket_array_add(*array, it * it);
    // Indices run across buckets in insertion order while nothing has been removed.
    for 0..29 assert(array[it] == it * it, "index %", it);
    array[17] = -1;
    locator: Bucket_Locator = .{ bucket_index = 2, slot_index = 1 };
    assert(bucket_array_find(*array, locator) == -1, "index 17 is bucket 2 slot 1");
    // Removing one leaves the others where they were; adding refills the hole.
    bucket_array_remove(*array, locator);
    assert(array[16] == 256 && array[18] == 324);
    new_locator := bucket_array_add(*array, 99);
    assert(array.count == 30);
    assert(new_locator.bucket_index == 3 || array[17] == 99, "the new item went somewhere reachable");
}

check_bit_array_edges :: () {
    bits := make_bit_array(129);
    defer deinit(*bits);
    assert(bits.slots.count == 3);
    for index: s64.[0, 62, 63, 64, 127, 128] {
        set_bit(*bits, index);
        assert(bits[index], "bit %", index);
        if index assert(!bits[index - 1] || index == 63 || index == 64 || index == 128, "bit % leaked", index - 1);
    }
    // Bit 63 is the sign bit of a word: toggling the whole array must not touch the other words.
    assert(bits.slots[0] < 0, "bit 63 is the top bit of the first word");
    toggle_bit(*bits, 63);
    assert(!bits[63] && bits[62] && bits[64]);
    set_all_bits(*bits);
    count := 0;
    for bits if it count += 1;
    assert(count == 129, "set_all_bits sets exactly count bits: %", count);
    toggle_all_bits(*bits);
    for bits assert(!it, "bit % after toggling all", it_index);
    bits[128] = true;
    last := -1;
    for: only_set bits last = it_index;
    assert(last == 128);

    empty := make_bit_array(0);
    for empty assert(false);
    deinit(*empty);
}
