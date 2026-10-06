// rules: ptr.1 ptr.4 ptr.5 ptr.6 ptr.7 ptr.8 ptr.9 ptr.10 ptr.11 ptr.12 ptr.14 ptr.16 ptr.17
// Self-checking program for the claims in docs/language/pointers-and-arrays.md.
#import "Basic";

first_of :: (p: *int) -> int {
    return p.*;
}

constant_names :: () -> [] string {
    return .["left", "right"];
}

read_past :: (v: [] int) -> int #no_abc {
    return v[v.count];
}

main :: () {
    // ptr.1: remove swaps in the last element, shrinks the count and revisits the slot.
    buf: [4] int = .[2, 4, 5, 7];
    values: [] int = buf;
    for v: values {
        if (v & 1) == 0 remove v;
    }
    assert(values.count == 2, "two even values removed");
    assert(buf[0] == 7 && buf[1] == 5 && buf[2] == 5 && buf[3] == 7, "removal order in the backing array");
    dyn: [..] int;
    for 1..5 array_add(*dyn, it);
    for dyn {
        if it == 2 remove it;
    }
    assert(dyn.count == 4 && dyn[1] == 5, "remove on a dynamic array moves the last element in");

    // ptr.4: pointer += scales by the element size.
    nums := int.[10, 20, 30, 40];
    p := *nums[0];
    p += 1;
    assert(p.* == 20, "p += 1 steps one element");
    assert(cast(u64) p - cast(u64) *nums[0] == 8, "one s64 element is 8 bytes");
    // ptr.5: pointer difference is an element count.
    q := *nums[3];
    assert(q - p == 2, "q - p counts elements");
    assert(q - *nums[0] == 3, "p - *buf[0] is an index");

    // ptr.6: null takes its type from context.
    np: *int = null;
    vp: *void = null;
    // ptr.7: !p and p == null test for null.
    assert(!np && np == null, "null pointer tests");
    assert(!vp, "null *void");
    np = *nums[2];
    assert(np != null && !(!np), "non-null pointer tests");

    // ptr.8: *T to *void implicitly, and a cast back.
    raw: *void = np;
    back := cast(*int) raw;
    assert(back.* == 30, "round trip through *void");

    // ptr.9: arr.data points at the first element and passes where a pointer is expected.
    #assert type_of(nums.data) == *int;
    assert(first_of(nums.data) == 10, "fixed array .data");
    assert(first_of(dyn.data) == 1, "dynamic array .data");

    // ptr.10: fixed array to view is implicit.
    view: [] int = nums;
    assert(view.count == 4 && view[3] == 40 && view.data == nums.data, "view over a fixed array");

    // ptr.11: views and dynamic arrays start with an s64 count, then the data pointer.
    #assert size_of([] int) == 16;
    #assert type_of(view.count) == s64;
    words := cast(*s64) *view;
    assert(words[0] == 4, "count comes first");
    assert(cast(*int) words[1] == nums.data, "data pointer comes second");
    dwords := cast(*s64) *dyn;
    assert(dwords[0] == dyn.count && cast(*int) dwords[1] == dyn.data, "dynamic array header");

    // ptr.12: by value with index, by pointer, and in reverse.
    sum := 0;
    for v, i: nums sum += v * i;
    assert(sum == 20 + 60 + 120, "for v, i");
    for *e: nums e.* += 1;
    assert(nums[0] == 11 && nums[3] == 41, "for *e writes through");
    order: [..] int;
    for < nums array_add(*order, it);
    assert(order[0] == 41 && order[3] == 11, "for < iterates in reverse");

    // ptr.14: #no_abc lets a view read past its count.
    backing := int.[1, 2, 3];
    short: [] int = backing;
    short.count = 2;
    assert(read_past(short) == 3, "#no_abc procedure reads past count");

    // ptr.16: a constant array literal returned as a view lives in writable global storage.
    names := constant_names();
    assert(names.count == 2 && names[1] == "right", "returned constant view");
    names[0] = "changed";
    assert(names[0] == "changed", "writes through a constant literal view");

    // ptr.17: an empty array literal's data is null.
    empty: [] int = .[];
    assert(empty.data == null && empty.count == 0, "empty literal view");

    print("ok\n");
}
