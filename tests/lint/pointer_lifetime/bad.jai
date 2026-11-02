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
    array_add(*xs, 2);
    sink(p);
}

cleanup_does_not_hide_growth :: () {
    xs: [..] int;
    array_add(*xs, 1);
    defer array_free(xs);
    p := *xs[0];
    array_add(*xs, 2);
    sink(p);
}
