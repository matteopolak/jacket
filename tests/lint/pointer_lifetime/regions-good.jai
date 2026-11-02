#import "Basic";

grow :: (xs: *[..] int) {} @GrowsArray
sink :: (p: *int) -> int {
    return p.*;
}

Box :: struct {
    p: *int;
    other: *int;
}

Inner :: struct {
    inner: [..] int;
}

grow_outer :: (xs: *[..] Inner) {} @GrowsArray

// A returned borrow belongs to the generation after this call's own growth.
fresh_return :: () {
    xs: [..] int;
    p := array_add(*xs);
    sink(p);
}

// Copying a descriptor does not make its storage part of the outer allocation.
nested_survives :: () {
    outer: [..] Inner;

    array_add(*outer);
    array_add(*outer[0].inner, 1);
    p := *outer[0].inner[0];

    grow_outer(*outer);
    sink(p);
}

replaced_copy :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];
    q := p;

    grow(*xs);
    q = *xs[0];
    sink(q);
}

field_precision :: () {
    xs: [..] int;

    array_add(*xs, 1);
    ys: [..] int;

    array_add(*ys, 1);
    box: Box;
    box.p = *xs[0];
    box.other = *ys[0];
    grow(*xs);
    sink(box.other);
}

// The copy has a different origin on one incoming path; must mode stays quiet.
branch_copy :: (flag: bool) {
    xs: [..] int;

    array_add(*xs, 1);
    ys: [..] int;

    array_add(*ys, 1);
    p := *xs[0];
    q: *int;

    if flag q = p;
    else q = *ys[0];

    grow(*xs);
    sink(q);
}

replaced_view :: () {
    xs: [..] int;

    array_add(*xs, 1);
    ys: [..] int;

    array_add(*ys, 1);
    v: [] int = xs;

    grow(*xs);
    v = ys;
    n := v[0];
}

// A growth annotation does not imply that a pointer result belongs to its argument.
independent :: (xs: *[..] int, p: *int) -> *int {
    return p;
} @GrowsArray
unrelated_result :: () {
    xs: [..] int;
    ys: [..] int;

    array_add(*ys, 1);
    p := independent(*xs, *ys[0]);

    grow(*xs);
    sink(p);
}

metadata_after_growth :: () {
    xs: [..] int;

    array_add(*xs, 1);
    v: [] int = xs;

    grow(*xs);
    count := v.count;

    v.data = xs.data;
    n := v[0];
}

// A struct and its first field may have the same machine address, but are distinct places.
first_field_is_a_pointer :: () {
    xs: [..] int;

    array_add(*xs, 1);
    ys: [..] int;

    array_add(*ys, 1);
    box: Box;
    box.p = *xs[0];
    box.other = *ys[0];
    grow(*xs);
    sink(box.other);
    box.p = *ys[0];
    sink(box.p);
}

rebound :: (xs: *[..] int) -> *int {
    other: [..] int;
    array_add(*other, 1);
    xs = *other;
    return xs.data;
} @GrowsArray

parameter_rebinding_does_not_invent_return_provenance :: () {
    xs: [..] int;
    p := rebound(*xs);
    grow(*xs);
    sink(p);
}

indirect_write_forgets_unresolved_aliases :: () {
    xs: [..] int;

    array_add(*xs, 1);
    ys: [..] int;

    array_add(*ys, 1);
    box := New(Box);
    box.p = *xs[0];
    copy := box;

    grow(*xs);
    copy.p = *ys[0];
    sink(box.p);
}

Alias :: union {
    p: *int;
    other: *int;
}

overlapping_fields :: () {
    xs: [..] int;

    array_add(*xs, 1);
    ys: [..] int;

    array_add(*ys, 1);
    a: Alias;
    a.p = *xs[0];
    grow(*xs);
    a.other = *ys[0];
    sink(a.p);
}

replace_field :: (box: *Box, p: *int) {
    box.p = p;
}

heap_field_call_can_rebind :: () {
    xs: [..] int;

    array_add(*xs, 1);
    ys: [..] int;

    array_add(*ys, 1);
    box := New(Box);
    box.p = *xs[0];
    replace_field(box, *ys[0]);
    grow(*xs);
    sink(box.p);
}

main :: () {}
