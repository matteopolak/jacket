// quick_sort from the compiler's own modules, with a comparison procedure.
#import "Basic";
#import "Sort";

main :: () {
    COUNT :: 3_000_000;
    values := NewArray(COUNT, int);

    state: u32 = 12345;

    for 0..COUNT - 1 {
        state = state * 1664525 + 1013904223;
        values[it] = cast(int) (state >> 8);
    }

    quick_sort(values, (a, b) => a - b);

    checksum := 0;
    for values checksum = (checksum * 31 + it) % 1_000_000_007;

    print("% %\n", values[COUNT / 2], checksum);
}
