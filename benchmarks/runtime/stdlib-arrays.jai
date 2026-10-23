// Basic's array helpers: array_add, array_find and unordered removal.
#import "Basic";

main :: () {
    checksum: u64 = 1469598103934665603;

    mix :: (value: int) #expand {
        checksum = (checksum ^ cast,no_check(u64) value) * 1099511628211;
    }

    for round: 0..39 {
        values: [..] int;
        for 0..200_000 - 1 array_add(*values, it * 31 + round);
        for 0..2_000 - 1 array_add(*values, it, it + 1, it + 2);

        for 0..299 {
            found, index := array_find(values, it * 9_973 + round);
            mix(ifx found then index else -1);
        }

        state: u32 = cast(u32) round;

        while values.count > 1_000 {
            state = state * 1664525 + 1013904223;
            array_unordered_remove_by_index(*values, cast(int) (state >> 8) % values.count);
        }

        for values mix(it);

        array_free(values);
    }

    print("%\n", checksum);
}
