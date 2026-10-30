#import "Basic";
S :: #import "Sort";

// Comparators that break the strict weak order must not make the sort leave the array or hang.
// Each of these gives answers that disagree with themselves, so the partition scans meet nonsense.

always_first :: (a: s64, b: s64) -> s64 {
    return -1;
}

always_last :: (a: s64, b: s64) -> s64 {
    return 1;
}

less_or_equal :: (a: s64, b: s64) -> s64 {
    return ifx a <= b then -1 else 1;
}

state: s64;

wandering :: (a: s64, b: s64) -> s64 {
    state = state * 1103515245 + 12345 + a;
    return ((state >> 16) & 0xff) % 3 - 1;
}

// Every value 0..count-1 is still there exactly once.
intact :: (items: [] s64) -> bool {
    seen := NewArray(items.count, bool,, temp);

    for items {
        if it < 0 || it >= items.count || seen[it] return false;
        seen[it] = true;
    }

    return true;
}

fill :: (items: [] s64, mode: s64) {
    for *items {
        if mode == 0 it.* = it_index;
        else if mode == 1 it.* = items.count - 1 - it_index;
        else it.* = (it_index * 7919) % items.count; // 7919 is prime and above every count used, so a permutation
    }
}

main :: () {
    for count: s64.[3, 17, 18, 50, 257, 4096] {
        for mode: 0..2 {
            for which: 0..3 {
                values := NewArray(count, s64,, temp);

                fill(values, mode);
                state = which + count;

                if which == 0 S.quick_sort(values, always_first);
                else if which == 1 S.quick_sort(values, always_last);
                else if which == 2 S.quick_sort(values, less_or_equal);
                else S.quick_sort(values, wandering);

                assert(intact(values), "an inconsistent comparator lost or duplicated an element");
            }
        }
    }

    print("ok\n");
}
