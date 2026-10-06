// libc calls back into #c_call procedures: under the interpreter they are reached through
// native thunks (see docs/compiler/interpreter.md).
#import "Basic";

libc :: #system_library "libc";
qsort :: (base: *void, count: u64, size: u64, compare: (*void, *void) -> s32 #c_call) #foreign libc;
bsearch :: (key: *void, base: *void, count: u64, size: u64, compare: (*void, *void) -> s32 #c_call) -> *void #foreign libc;

by_value :: (a: *void, b: *void) -> s32 #c_call {
    x, y := (cast(*s32) a).*, (cast(*s32) b).*;
    return cast(s32) (x > y) - cast(s32) (x < y);
}

compared := 0;
counting :: (a: *void, b: *void) -> s32 #c_call {
    compared += 1; // Globals are shared with the interpreter.
    return by_value(a, b);
}

// The WASM target (the browser playground) has no C library to call back from, so there the
// same #c_call procedures are driven from Jai instead (the libc calls are still compiled).
sort_in_jai :: (xs: [] s32, compare: (*void, *void) -> s32 #c_call) {
    for i: 1..xs.count - 1 {
        j := i;
        while j > 0 && compare(*xs[j - 1], *xs[j]) > 0 {
            xs[j - 1], xs[j] = xs[j], xs[j - 1];
            j -= 1;
        }
    }
}

search_in_jai :: (key: *s32, xs: [] s32, compare: (*void, *void) -> s32 #c_call) -> *void {
    low, high := 0, xs.count;
    while low < high {
        middle := (low + high) / 2;
        order := compare(key, *xs[middle]);
        if order == 0 return *xs[middle];
        if order < 0 high = middle;
        else low = middle + 1;
    }
    return null;
}

main :: () {
    xs := s32.[5, 3, 9, 1, 7];
    if OS == .WASM sort_in_jai(xs, by_value);
    else qsort(xs.data, xs.count, size_of(s32), by_value);
    print("%\n", xs);
    key: s32 = 7;
    find :: (key: *s32, xs: [] s32) -> *s32 {
        if OS == .WASM return cast(*s32) search_in_jai(key, xs, counting);
        return cast(*s32) bsearch(key, xs.data, cast(u64) xs.count, size_of(s32), counting);
    }
    found := find(*key, xs);
    print("% at index %\n", found.*, found - xs.data);
    assert(compared > 0);
    assert(find(*key, xs) == found);
}
