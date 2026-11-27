#import "Basic";

// Atomic operations read and write integers in the program's byte order, at every width.
main :: () {
    check(cast(u16) 0x0102);
    check(cast(u32) 0x01020304);
    check(cast(u64) 0x0102030405060708);
}

check :: (start: $T) {
    SEQ_CST :: 4;
    x: T = start;
    before := atomic_fetch_add_order(*x, 0x0100, SEQ_CST);
    added := x;

    atomic_fetch_sub_order(*x, 0x0100, SEQ_CST);
    atomic_fetch_or_order(*x, 0x00F0, SEQ_CST);
    ored := x;

    atomic_fetch_and_order(*x, cast,trunc(T) ~0x00F0, SEQ_CST);
    atomic_fetch_xor_order(*x, 0x0F00, SEQ_CST);
    xored := x;
    old := atomic_exchange(*x, start);
    swapped := compare_and_swap(*x, start, 7);
    loaded := atomic_load(*x);

    atomic_store(*x, 9);
    print("% % % % % % % %\n", formatInt(before, base = 16), formatInt(added, base = 16),
        formatInt(ored, base = 16), formatInt(xored, base = 16), formatInt(old, base = 16),
        swapped, loaded, x);
}

atomic_fetch_add_order :: (pointer: *$T, value: T, order: s64) -> T #intrinsic;
atomic_fetch_sub_order :: (pointer: *$T, value: T, order: s64) -> T #intrinsic;
atomic_fetch_and_order :: (pointer: *$T, value: T, order: s64) -> T #intrinsic;
atomic_fetch_or_order :: (pointer: *$T, value: T, order: s64) -> T #intrinsic;
atomic_fetch_xor_order :: (pointer: *$T, value: T, order: s64) -> T #intrinsic;
