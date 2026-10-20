// A merge sort written here, so no library code is timed. Prints the median and a checksum.
#import "Basic";

merge_sort :: (values: [] int, scratch: [] int) {
    if values.count < 2 return;

    middle := values.count / 2;

    merge_sort(array_view(values, 0, middle), array_view(scratch, 0, middle));
    merge_sort(array_view(values, middle), array_view(scratch, middle));

    i, j, k := 0, middle, 0;
    while i < middle && j < values.count {
        if values[i] <= values[j] {
            scratch[k] = values[i];
            i += 1;
        } else {
            scratch[k] = values[j];
            j += 1;
        }

        k += 1;
    }

    while i < middle {
        scratch[k] = values[i];
        i += 1;
        k += 1;
    }

    while j < values.count {
        scratch[k] = values[j];
        j += 1;
        k += 1;
    }

    for 0..values.count - 1 values[it] = scratch[it];
}

main :: () {
    COUNT :: 3_000_000;
    values := NewArray(COUNT, int);
    scratch := NewArray(COUNT, int);

    state: u32 = 12345;

    for 0..COUNT - 1 {
        state = state * 1664525 + 1013904223;
        values[it] = cast(int) (state >> 8);
    }

    merge_sort(values, scratch);

    checksum := 0;
    for values checksum = (checksum * 31 + it) % 1_000_000_007;

    print("% %\n", values[COUNT / 2], checksum);
}
