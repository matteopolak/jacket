// array_find over arrays long enough for the block search, for every element size and kind.
#import "Basic";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    if failures < 20 print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

Color :: enum u16 {
    RED :: 1;
    GREEN :: 2;
    BLUE :: 300;
}

// True for the item that is 14 less than the negation of the one asked for: -7 asks for 7.
negated_plus_fourteen :: (value: s32, item: s32) -> bool {
    return value == -item;
}

naive :: (array: [] $T, item: T) -> bool, s64 {
    for array if it == item return true, it_index;
    return false, -1;
}

try :: ($T: Type, sizes: [] int, absent: T, make: (int) -> T) {
    for size: sizes {
        array: [..] T;
        for 0..size - 1 array_add(*array, make(it));

        // Items present at every position that matters, and absent ones.
        for position: s64.[0, 1, 15, 16, 31, 32, 33, 63, 64, 127, 128, 129, 255, 256, size - 1, size / 2] {
            if position < 0 || position >= size continue;
            found, index := array_find(array, array[position]);
            expected_found, expected := naive(array, array[position]);
            check(found == expected_found && index == expected, tprint("% of %: found % at %, expected %", T, size, found, index, expected));
        }

        found, index := array_find(array, absent);

        check(!found && index == -1, tprint("% of %: absent item found", T, size));
        array_free(array);
    }
}

main :: () {
    sizes := int.[0, 1, 31, 32, 33, 100, 128, 129, 1000, 4099];

    try(u8, sizes, 252, (i) => cast(u8) (i * 7 % 251));
    try(s8, sizes, -5, (i) => cast(s8) (i % 120));
    try(u16, sizes, 1, (i) => cast(u16) (i * 3 + 3));
    try(s32, sizes, 3, (i) => cast(s32) (i * 5 - 1000));
    try(u32, sizes, 7, (i) => cast(u32) (i * 11 + 11));
    try(s64, sizes, 9, (i) => cast(s64) i * 1_000_000_007 - 5_000_000_000);
    try(u64, sizes, 9, (i) => cast(u64) i * 40_000_000_001 + 40_000_000_001);
    try(Color, sizes, cast(Color) 7, (i) => ifx i % 3 == 0 then Color.RED else ifx i % 3 == 1 then Color.GREEN else Color.BLUE);

    // Pointers compare by address.
    storage: [200] int;
    pointers: [..] *int;

    for 0..199 array_add(*pointers, *storage[it]);

    for 0..199 {
        found, index := array_find(pointers, *storage[it]);
        check(found && index == it, "pointer item");
    }

    found, index := array_find(pointers, null);

    check(!found, "null pointer is absent");

    // A custom comparison is used, whatever the length.
    values: [..] s32;
    for 0..999 array_add(*values, cast(s32) it);

    found, index = array_find(values, -7, negated_plus_fourteen);
    check(found && index == 7, tprint("custom comparison found % at %", found, index));

    // Floats compare as numbers, not bits: -0 equals 0, so they take the item-by-item path.
    floats: [..] float32;
    for 1..100 array_add(*floats, cast(float32) it);

    floats[60] = -0.0;
    found, index = array_find(floats, 0.0);
    check(found && index == 60, tprint("float zero found % at %", found, index));
    found, index = array_find(floats, 50.0);
    check(found && index == 49, "float 50");

    if failures {
        print("% failures\n", failures);
        assert(false, "tests failed");
    }

    write_string("ok\n");
}
