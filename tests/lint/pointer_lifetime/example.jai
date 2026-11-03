#import "Basic";

main :: () {
    xs: [..] int;

    defer array_free(xs);

    array_add(*xs, 1);
    xs.count = xs.allocated;
    p := *xs[0];

    array_add(*xs, 2);

    print("%\n", p.*);
}
