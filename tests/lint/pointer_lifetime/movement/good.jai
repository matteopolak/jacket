#import "Basic";

sink :: (p: *int) {
    print("%", p.*);
}

ordered :: (a: *[..] int, i: int) {} @RemovesFromArray(ordered,1)
unordered :: (a: *[..] int, i: int) {} @RemovesFromArray(unordered,1)
sort :: (a: *[..] int) {} @MovesElements(sort)
swap_at :: (a: *[..] int, i: int, j: int) {} @MovesElements(swap,1,2)

prefix :: () {
    a: [..] int;
    p := *a[0];
    ordered(*a, 2);
    sink(p);
}

unaffected :: () {
    a: [..] int;
    p := *a[3];
    unordered(*a, 0);
    sink(p);
}

unknown_position :: (i: int) {
    a: [..] int;
    p := *a[i];
    ordered(*a, 0);
    sink(p);
}

conditional :: (b: bool) {
    a: [..] int;
    p := *a[3];
    if b ordered(*a, 0);
    sink(p);
}

unknown_sort :: () {
    a: [..] int;
    p := *a[3];
    sort(*a);
    sink(p);
}

same_slot_swap :: () {
    a: [..] int;
    p := *a[3];
    swap_at(*a, 3, 3);
    sink(p);
}

count_grow :: () {
    a: [..] int;
    p := *a[3];
    a.count = 8;
    sink(p);
}

rederive :: () {
    a: [..] int;
    p := *a[3];

    ordered(*a, 1);
    p = *a[3];
    sink(p);
}

offset_is_a_different_element :: () {
    a: [..] int;
    p := *a[0];
    q := p + 1;

    unordered(*a, 0);
    sink(q);
}

unknown_swap_equality :: (i: int, j: int) {
    a: [..] int;
    p := *a[i];
    swap_at(*a, i, j);
    sink(p);
}

stale_last :: () {
    a: [..] int;
    p := *a[a.count - 1];

    a.count += 1;
    unordered(*a, 0);
    sink(p);
}

shrink_local_header :: (v: [] int) {
    local := v;
    local.count = 0;
}

by_value_header :: () {
    a: [..] int;
    p := *a[0];
    shrink_local_header(a);
    sink(p);
}
