#import "Basic";

counted_growth :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    for 1..100 array_add(*xs, it);

    print("%", p.*);
}

empty_range :: () {
    xs: [..] int;
    p := *xs[0];
    for 1..0 array_add(*xs, it);
    print("%", p.*);
}

few_adds :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    for 1..2 array_add(*xs, it);

    print("%", p.*);
}

break_early :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    for 1..100 {
        array_add(*xs, it);
        break;
    }

    print("%", p.*);
}

changed_counter :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];

    for 1..100 {
        array_add(*xs, it);
        it = 100;
    }

    print("%", p.*);
}
