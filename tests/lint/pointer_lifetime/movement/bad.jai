#import "Basic";

sink :: (p: *int) {
    print("%", p.*);
}

ordered :: (a: *[..] int, i: int) {} @RemovesFromArray(ordered,1)
unordered :: (a: *[..] int, i: int) {} @RemovesFromArray(unordered,1)
insert_at :: (a: *[..] int, i: int) {} @MovesElements(insert,1)
reset :: (a: *[..] int) {} @ResetsElements
swap_at :: (a: *[..] int, i: int, j: int) {} @MovesElements(swap,1,2)
wrap :: (a: *[..] int, i: int) {
    ordered(a, i);
}

ordered_suffix :: () {
    a: [..] int;
    p := *a[3];
    ordered(*a, 1);
    sink(p);
}

unordered_slot :: () {
    a: [..] int;
    p := *a[0];
    unordered(*a, 0);
    sink(p);
}

unordered_last :: () {
    a: [..] int;
    p := *a[a.count - 1];
    unordered(*a, 0);
    sink(p);
}

insert_suffix :: () {
    a: [..] int;
    p := *a[3];
    insert_at(*a, 1);
    sink(p);
}

reset_copy :: () {
    a: [..] int;
    p := *a[0];
    q := p;

    reset(*a);
    sink(q);
}

count_shrink :: () {
    a: [..] int;
    p := *a[3];
    a.count = 2;
    sink(p);
}

swap_endpoint :: () {
    a: [..] int;
    p := *a[3];
    swap_at(*a, 1, 3);
    sink(p);
}

wrapper :: () {
    a: [..] int;
    p := *a[3];
    wrap(*a, 1);
    sink(p);
}

cut :: (b: bool) {
    a: [..] int;
    p := *a[3];

    if b ordered(*a, 1);
    else reset(*a);

    sink(p);
}

stdlib_pop :: () {
    a: [..] int;
    p := *a[a.count - 1];
    pop(*a);
    sink(p);
}

stdlib_remove :: () {
    a: [..] int;
    p := *a[3];
    array_ordered_remove_by_index(*a, 1);
    sink(p);
}

count_delta :: () {
    a: [..] int;
    p := *a[a.count - 1];
    a.count -= 1;
    sink(p);
}

stdlib_reset :: () {
    a: [..] int;
    p := *a[0];
    array_reset_keeping_memory(*a);
    sink(p);
}
