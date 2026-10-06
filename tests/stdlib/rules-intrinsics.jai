// rules: intrin.1 intrin.3 intrin.4 intrin.5 intrin.6
// `#intrinsic` procedures: declared without a body, lowered by name to a compiler operation.
#import "Basic";

// intrin.1: a user declaration with `#intrinsic` and no body is lowered by its name.
sqrt :: (x: float64) -> float64 #intrinsic;

// intrin.5: intrinsics take no implicit context, so a #c_call procedure (which has none)
// may call them directly.
copy_without_context :: (dest: *void, source: *void, count: s64) #c_call {
    memcpy(dest, source, count);
}

main :: () {
    // intrin.1
    assert(sqrt(81.0) == 9.0, "sqrt intrinsic");

    // intrin.3: compare_and_swap stores only when the old value matches and returns
    // whether it did plus the previous value.
    x: s64 = 5;
    ok, old := compare_and_swap(*x, 5, 9);
    assert(ok && old == 5 && x == 9, "first compare_and_swap");
    ok, old = compare_and_swap(*x, 5, 1);
    assert(!ok && old == 9 && x == 9, "second compare_and_swap");
    // The operand width follows the value type.
    small: u8 = 200;
    ok8, old8 := compare_and_swap(*small, 200, 7);
    assert(ok8 && old8 == 200 && small == 7, "u8 compare_and_swap");
    half: s16 = -3;
    ok16, old16 := compare_and_swap(*half, -3, 300);
    assert(ok16 && old16 == -3 && half == 300, "s16 compare_and_swap");
    word: u32 = 0xdead_beef;
    ok32, old32 := compare_and_swap(*word, 1, 2);
    assert(!ok32 && old32 == 0xdead_beef && word == 0xdead_beef, "u32 compare_and_swap");

    // intrin.4: memset, memcpy and memcmp.
    a: [8] u8;
    b: [8] u8;
    memset(a.data, 7, 8);
    for a assert(it == 7, "memset");
    memcpy(b.data, a.data, 8);
    assert(memcmp(a.data, b.data, 8) == 0, "memcmp equal");
    memset(b.data, 0, 8);
    assert(memcmp(a.data, b.data, 8) > 0, "memcmp greater");
    assert(memcmp(b.data, a.data, 8) < 0, "memcmp less");
    b[0] = 7;
    b[1] = 9;
    assert(memcmp(a.data, b.data, 8) < 0, "memcmp: the first differing byte decides");
    assert(memcmp(a.data, b.data, 1) == 0, "memcmp stops at count");

    // intrin.5
    c: [4] u8;
    copy_without_context(c.data, a.data, 4);
    for c assert(it == 7, "memcpy from #c_call");

    // intrin.6: memcpy, memset and memcmp can be used as procedure values.
    set_proc := memset;
    copy_proc := memcpy;
    cmp_proc := memcmp;
    d: [4] u8;
    set_proc(d.data, 3, 4);
    for d assert(it == 3, "memset through a value");
    copy_proc(c.data, d.data, 2);
    assert(c[0] == 3 && c[1] == 3 && c[2] == 7, "memcpy through a value");
    assert(cmp_proc(c.data, d.data, 2) == 0 && cmp_proc(c.data, d.data, 4) > 0, "memcmp through a value");

    print("ok\n");
}
