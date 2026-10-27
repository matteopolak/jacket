// quick_sort with a comparison procedure over integers and strings, and intro_sort over
// integers. Each iteration builds its input with a small generator and sorts it.
#import "Basic";
#import "Sort";
#import "IntroSort";
#import "Extensions/Testing";

SORT_INTS :: 3_000_000;
SORT_WORDS :: 300_000;
SORT_NUMBERS :: 2_000_000;

quick_sort_ints :: (b: *Bench) {
    set_items(b, SORT_INTS);

    for 0..b.iterations - 1 {
        values := NewArray(SORT_INTS, int);
        state: u32 = 12345;

        for 0..SORT_INTS - 1 {
            state = state * 1664525 + 1013904223;
            values[it] = cast(int) (state >> 8);
        }

        quick_sort(values, (x, y) => x - y);

        black_box(values[SORT_INTS / 2]);
        array_free(values);
    }
} @bench @samples(10)

quick_sort_strings :: (b: *Bench) {
    set_items(b, SORT_WORDS);

    for 0..b.iterations - 1 {
        buffer := cast(*u8) alloc(SORT_WORDS * 12);
        words := NewArray(SORT_WORDS, string);
        state: u32 = 99;

        for i: 0..SORT_WORDS - 1 {
            word := string.{ 4 + i % 8, buffer + i * 12 };

            for j: 0..word.count - 1 {
                state = state * 1664525 + 1013904223;
                word.data[j] = cast(u8) (#char "a" + (state >> 24) % 4);
            }

            words[i] = word;
        }

        quick_sort(words, compare_strings);

        black_box(words[SORT_WORDS / 2]);
        array_free(words);
        free(buffer);
    }
} @bench @samples(10)

intro_sort_ints :: (b: *Bench) {
    set_items(b, SORT_NUMBERS);

    for 0..b.iterations - 1 {
        values := NewArray(SORT_NUMBERS, int);
        state: u32 = 99;

        for 0..SORT_NUMBERS - 1 {
            state = state * 1664525 + 1013904223;
            values[it] = cast(int) (state >> 4);
        }

        intro_sort(values, (x, y) => x - y);

        black_box(values[SORT_NUMBERS / 2]);
        array_free(values);
    }
} @bench @samples(10)
