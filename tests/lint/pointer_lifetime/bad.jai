#import "Basic";

grow :: (xs: *[..] int) {} @GrowsArray
sink :: (p: *int) -> int {
    return p.*;
}

straight :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    grow(*xs);
    sink(p);
}

both_branches :: (flag: bool) {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    if flag grow(*xs);
    else grow(*xs);

    sink(p);
}

loop_body :: (flag: bool) {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    while flag {
        grow(*xs);
        sink(p);
    }
}

pointer_loop :: () {
    xs: [..] int;

    array_add(*xs, 1);

    for *xs {
        grow(*xs);
        sink(it);
    }
}

named_loop :: () {
    xs: [..] int;

    array_add(*xs, 1);

    for *item: xs {
        grow(*xs);
        sink(item);
    }
}

// The pointer is carried over the backedge; every exit has grown the array.
loop_carried :: (flag: bool) {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    while true {
        grow(*xs);
        if flag break;
        continue;
    }

    sink(p);
}

main :: () {
    straight();
    both_branches(true);
    loop_body(true);
    pointer_loop();
    named_loop();
    loop_carried(true);
}

// A library note, including polymorphic overload selection, drives the same effect.
library_call :: () {
    xs: [..] int;

    array_add(*xs, 1);
    array_add(*xs, 1);
    p := *xs[0];

    xs.count = xs.allocated;
    array_add(*xs, 2);
    sink(p);
}

cleanup_does_not_hide_growth :: () {
    xs: [..] int;

    array_add(*xs, 1);
    defer array_free(xs);
    p := *xs[0];

    xs.count = xs.allocated;
    array_add(*xs, 2);
    sink(p);
}

// Checked instantiations keep different static choices at the same source span.
per_instance :: ($INVALIDATE: bool) {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    #if INVALIDATE grow(*xs);
    sink(p);
}

instances :: () {
    per_instance(true);
    per_instance(false);
}

named_argument :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    grow(xs = *xs);
    sink(p);
}

grow_many :: (xs: *[..] int, rest: ..int) {} @GrowsArray

spread_arguments :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];
    rest: [2] int;

    grow_many(*xs, ..rest);
    sink(p);
}

switch_cut :: (choice: int) {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    if choice == {
        case 1; grow(*xs);
        case; grow(*xs);
    }

    sink(p);
}

cleanup_edge :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    {
        defer grow(*xs);
    }

    sink(p);
}

labelled_exit :: (flag: bool) {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    while: outer true {
        grow(*xs);

        while flag {
            break outer;
        }

        break;
    }

    sink(p);
}

empty_expansion :: () #expand {}

expanded_caller :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    empty_expansion();
    grow(*xs);
    sink(p);
}

UsingOptions :: struct {
    enabled: bool;
}

using_body :: (using options: UsingOptions) {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    grow(*xs);
    if enabled sink(p);
}

inserted_caller :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    #insert "grow(*xs);";
    sink(p);
}
