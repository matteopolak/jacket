#import "Basic";

add_guest :: (xs: *[..] int, invited: bool) {
    if invited array_add(xs, 1);
}

conditional_wrapper :: (invited: bool) {
    xs: [..] int;
    array_add(*xs, 1);
    p := *xs[0];
    add_guest(*xs, invited);
    print("%", p.*);
}
