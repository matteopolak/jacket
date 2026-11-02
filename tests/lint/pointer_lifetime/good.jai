#import "Basic";

grow :: (xs: *[..] int) {} @GrowsArray
sink :: (p: *int) -> int {
    return p.*;
}

one_branch :: (flag: bool) {
    xs: [..] int;
    array_add(*xs, 1);
    p := *xs[0];
    if flag grow(*xs);
    sink(p);
}

correlated :: (flag: bool) {
    xs: [..] int;
    array_add(*xs, 1);
    p := *xs[0];
    if flag grow(*xs);
    if !flag sink(p);
}

rederived :: () {
    xs: [..] int;
    array_add(*xs, 1);
    p := *xs[0];
    grow(*xs);
    p = *xs[0];
    sink(p);
}

other_region :: () {
    xs: [..] int;
    array_add(*xs, 1);
    ys: [..] int;
    p := *xs[0];
    grow(*ys);
    sink(p);
}

zero_iterations :: (flag: bool) {
    xs: [..] int;
    array_add(*xs, 1);
    p := *xs[0];
    while flag grow(*xs);
    sink(p);
}

first_iteration_live :: (flag: bool) {
    xs: [..] int;
    array_add(*xs, 1);
    p := *xs[0];
    while flag {
        sink(p);
        grow(*xs);
    }
}

exiting_branch :: (flag: bool) {
    xs: [..] int;
    array_add(*xs, 1);
    p := *xs[0];
    if flag {
        grow(*xs);
        return;
    }
    sink(p);
}

rederived_in_loop :: () {
    xs: [..] int;
    array_add(*xs, 1);
    for *xs {
        grow(*xs);
        it = *xs[0];
        sink(it);
    }
}

shadowed :: () {
    xs: [..] int;
    array_add(*xs, 1);
    p := *xs[0];
    grow(*xs);
    {
        p: *int;
        sink(p);
    }
}

main :: () {
    one_branch(true);
    correlated(true);
    rederived();
    other_region();
    zero_iterations(false);
    first_iteration_live(false);
    exiting_branch(false);
    rederived_in_loop();
    shadowed();
}

compare_only :: () {
    xs: [..] int;
    array_add(*xs, 1);
    p := *xs[0];
    grow(*xs);
    if p != null {}
}

region_replaced :: () {
    xs: [..] int;
    array_add(*xs, 1);
    ys: [..] int;
    p := *xs[0];
    xs = ys;
    grow(*xs);
    sink(p);
}

storage_field_replaced :: () {
    xs: [..] int;
    array_add(*xs, 1);
    ys: [..] int;
    p := *xs[0];
    xs.data = ys.data;
    grow(*xs);
    sink(p);
}

// Exposed locals can be changed through aliases; do not identify different storage as xs.
array_alias :: () {
    xs: [..] int;
    array_add(*xs, 1);
    ys: [..] int;
    alias := *xs;
    p := *xs[0];
    alias.* = ys;
    grow(*xs);
    sink(p);
}

pointer_alias :: () {
    xs: [..] int;
    array_add(*xs, 1);
    p := *xs[0];
    alias := *p;
    grow(*xs);
    alias.* = *xs[0];
    sink(p);
}

unevaluated_type_query :: () {
    xs: [..] int;
    array_add(*xs, 1);
    p := *xs[0];
    grow(*xs);
    size := size_of(type_of(p));
}

cleanup_with_rederivation :: () {
    xs: [..] int;
    array_add(*xs, 1);
    defer array_free(xs);
    p := *xs[0];
    array_add(*xs, 2);
    p = *xs[0];
    sink(p);
}

// A cleanup may replace the descriptor, so this body needs explicit cleanup edges.
unsupported_cleanup :: () {
    xs: [..] int;
    array_add(*xs, 1);
    ys: [..] int;
    p := *xs[0];
    {
        defer {
            xs = ys;
        }
    }
    grow(*xs);
    sink(p);
}

// A one-pass walk sees an invalid first use. The backedge contributes a fresh pointer,
// so this use site is not invalid on every path and must stay quiet.
backedge_restores_pointer :: (flag: bool) {
    xs: [..] int;
    array_add(*xs, 1);
    p := *xs[0];
    grow(*xs);
    while flag {
        sink(p);
        p = *xs[0];
    }
}
