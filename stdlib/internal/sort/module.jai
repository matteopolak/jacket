// Sorts an array view in place with a three-way comparator: negative, zero or positive.
// Quicksort partitions around a median of three; a range that nests too deep falls back to
// heap_sort, and short ranges finish with insertion_sort.
intro_sort :: (array: [] $T, f: (T, T) -> $R) {
    if array.count < 2 return;

    depth_budget: s64;
    remaining := array.count;

    while remaining > 1 {
        depth_budget += 2;
        remaining /= 2;
    }

    sort_range(array.data, array.count, depth_budget, f);
}

// Sorts by a key computed once per element. Elements with equal keys keep their order.
sort_by_key :: (array: [] $T, key: (T) -> $R) {
    if array.count < 2 return;

    Keyed :: struct {
        key: R;
        position: s64;
        value: T;
    }

    by_key :: (a: Keyed, b: Keyed) -> s64 {
        if a.key < b.key return -1;
        if b.key < a.key return 1;
        if a.position < b.position return -1;
        if a.position > b.position return 1;

        return 0;
    }

    pairs := cast(*Keyed) alloc(array.count * size_of(Keyed));

    defer free(pairs);

    for index: 0..array.count - 1 {
        pairs[index].key = key(array[index]);
        pairs[index].position = index;
        pairs[index].value = array[index];
    }

    intro_sort(view(pairs, array.count), by_key);

    for index: 0..array.count - 1 array[index] = pairs[index].value;
}

heap_sort :: (array: [] $T, f: (T, T) -> $R) -> [] T {
    count := array.count;
    i := count / 2;

    while i > 0 {
        i -= 1;
        sift(array, i, count, f);
    }

    while count > 1 {
        count -= 1;
        swap(array.data, array.data + count);
        sift(array, 0, count, f);
    }

    return array;
}

insertion_sort :: (array: [] $T, f: (T, T) -> $R) {
    for i: 1..array.count - 1 {
        value := array[i];
        j := i;

        while j > 0 && f(array[j - 1], value) > 0 {
            array[j] = array[j - 1];
            j -= 1;
        }

        array[j] = value;
    }
}

quick_sort :: (array: [] $T, f: (T, T) -> $R) -> [] T {
    intro_sort(array, f);
    return array;
}

quick_sort :: (array: [] $T, f: (T) -> $R) -> [] T {
    sort_by_key(array, f);
    return array;
}

compare_floats :: (a: float, b: float) -> s64 {
    // NaN is placed after every number and ties only with other NaNs, so the order is total.
    a_nan := a != a;
    b_nan := b != b;

    if a_nan return ifx b_nan then 0 else 1;
    if b_nan return -1;

    if a < b return -1;
    if a > b return 1;

    return 0;
}

compare_strings :: (a: string, b: string) -> s64 {
    return Strings.compare(a, b);
}

#scope_file
#import "Basic";
Strings :: #import "String";

SHORT_RANGE :: 16;

view :: (data: *$T, count: s64) -> [] T {
    result: [] T;
    result.data = data;
    result.count = count;
    return result;
}

sift :: (array: [] $T, root: s64, count: s64, f: (T, T) -> $R) {
    index := root;
    while index * 2 + 1 < count {
        child := index * 2 + 1;
        if child + 1 < count && f(array[child], array[child + 1]) < 0 child += 1;
        if f(array[index], array[child]) >= 0 break;

        swap(array.data + index, array.data + child);
        index = child;
    }
}

sort_range :: (start: *$T, length: s64, depth_budget: s64, f: (T, T) -> $R) {
    data := start;
    count := length;
    budget := depth_budget;

    while count > SHORT_RANGE {
        if budget == 0 {
            heap_sort(view(data, count), f);
            return;
        }

        budget -= 1;
        split := partition(data, count, f);

        // Recurse into the smaller side, loop over the larger one.
        if split < count - split - 1 {
            sort_range(data, split, budget, f);
            data += split + 1;
            count -= split + 1;
        } else {
            sort_range(data + split + 1, count - split - 1, budget, f);
            count = split;
        }
    }

    insertion_sort(view(data, count), f);
}

// Needs count >= 3. Returns the final index of the pivot; no later element is smaller than it.
partition :: (data: *$T, count: s64, f: (T, T) -> $R) -> s64 {
    middle := count / 2;
    last := count - 1;

    if f(data[middle], data[0]) < 0 swap(data + middle, data);
    if f(data[last], data[0]) < 0 swap(data + last, data);
    if f(data[last], data[middle]) < 0 swap(data + last, data + middle);

    // The median sits beside the end. The scans also stop at the array ends, so an inconsistent comparator
    // can only give a poor split, never an access outside the range.
    swap(data + middle, data + last - 1);
    pivot := data[last - 1];

    low := 0;
    high := last - 1;

    while true {
        low += 1;
        while low < last - 1 && f(data[low], pivot) < 0 low += 1;

        high -= 1;
        while high > 0 && f(pivot, data[high]) < 0 high -= 1;
        if low >= high break;

        swap(data + low, data + high);
    }

    swap(data + low, data + last - 1);

    return low;
}
