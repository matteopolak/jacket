// Sorting strings with quick_sort and compare_strings, and integers with intro_sort.
#import "Basic";
#import "Sort";
#import "IntroSort";

main :: () {
    COUNT :: 300_000;
    buffer := cast(*u8) alloc(COUNT * 12);
    words := NewArray(COUNT, string);
    state: u32 = 99;

    for i: 0..COUNT - 1 {
        word := string.{ 4 + i % 8, buffer + i * 12 };

        for j: 0..word.count - 1 {
            state = state * 1664525 + 1013904223;
            word.data[j] = cast(u8) (#char "a" + (state >> 24) % 4);
        }

        words[i] = word;
    }

    quick_sort(words, compare_strings);

    checksum: u64 = 1469598103934665603;
    for words checksum = (checksum ^ cast(u64) it.count ^ it[it.count - 1]) * 1099511628211;

    NUMBERS :: 2_000_000;
    values := NewArray(NUMBERS, int);

    for 0..NUMBERS - 1 {
        state = state * 1664525 + 1013904223;
        values[it] = cast(int) (state >> 4);
    }

    intro_sort(values, (a, b) => a - b);

    for values checksum = (checksum ^ cast(u64) it) * 1099511628211;

    print("% % %\n", words[COUNT / 2], values[NUMBERS / 2], checksum);
}
