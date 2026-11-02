Targets :: #import "Targets";

sink :: (p: *int) -> int {
    return p.*;
}

probe :: () {
    xs: [..] int;
    p := *xs[0];
    f := Targets.factory();

    f(*xs);
    sink(p);
}
