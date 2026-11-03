#import "Basic";

main :: () {
    xs: [..] int;

    array_add(*xs, 1, 2, 2, 3);

    for xs {
        if it == 2 remove it;
    }

    print("%\n", xs);
}
