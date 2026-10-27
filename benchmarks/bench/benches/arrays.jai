// Basic's array helpers: array_add, array_find and unordered removal.
#import "Basic";
#import "Extensions/Testing";

array_values: [..] int;

fill_array :: (values: *[..] int, round: int) {
    for 0..200_000 - 1 array_add(values, it * 31 + round);
    for 0..2_000 - 1 array_add(values, it, it + 1, it + 2);
}

// 200 000 single items and 2 000 groups of three.
array_add_items :: (b: *Bench) {
    set_items(b, 206_000);

    for round: 0..b.iterations - 1 {
        values: [..] int;

        fill_array(*values, round);
        black_box(values.count);
        array_free(values);
    }
} @bench @samples(20)

// 300 searches through an array of 206 000 integers, most of them misses.
array_find_values :: (b: *Bench) {
    if !array_values.count fill_array(*array_values, 0);

    reset_timer(b);
    set_items(b, 300);

    for round: 0..b.iterations - 1 {
        checksum := 0;

        for 0..299 {
            found, index := array_find(array_values, it * 9_973 + round % 40);
            checksum += ifx found then index else -1;
        }

        black_box(checksum);
    }
} @bench @samples(20)

// Fills the array, then removes random elements until 1 000 are left.
array_remove_unordered :: (b: *Bench) {
    for round: 0..b.iterations - 1 {
        values: [..] int;

        fill_array(*values, round % 40);

        state: u32 = cast(u32) (round % 40);

        while values.count > 1_000 {
            state = state * 1664525 + 1013904223;
            array_unordered_remove_by_index(*values, cast(int) (state >> 8) % values.count);
        }

        black_box(values.count);
        array_free(values);
    }
} @bench @samples(10)
