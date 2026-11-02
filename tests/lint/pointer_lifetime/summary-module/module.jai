#scope_export
#import "Basic";

grow :: (xs: *[..] int) {} @GrowsArray
push :: (xs: *[..] int) {
    grow(xs);
}

// Checked without a program/caller: the selected callee summary still applies.
probe :: () -> int {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    push(*xs);

    return p.*;
}
