#import "Basic";

reserved :: () {
    xs: [..] int;

    array_reserve(*xs, 4);
    p := *xs[0];

    array_add(*xs, 1);
    print("%", p.*);
}

second_add :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    array_add(*xs, 2);
    print("%", p.*);
}

count_changed :: () {
    xs: [..] int;

    array_reserve(*xs, 4);
    xs.count = 4;
    p := *xs[0];

    array_add(*xs, 2);
    print("%", p.*);
}
