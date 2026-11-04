#import "Basic";

add_inner :: (xs: *[..] int) {
    array_add(xs, 1);
}

add_outer :: (xs: *[..] int) {
    add_inner(xs);
}

probe :: () {
    xs: [..] int;
    p := *xs[0];
    add_outer(*xs);
    print("%", p.*);
}
