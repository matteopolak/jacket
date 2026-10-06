#import "Basic";

main :: () {
    xs: [..] int;
    array_add(*xs, 1, 2, 2, 3);
    for xs {
        if it == 2 remove it;
    }
    for < xs {
        if it == 3 array_ordered_remove_by_index(*xs, it_index);
    }
    for xs {
        if it == 1 {
            array_unordered_remove_by_index(*xs, it_index);
            break;
        }
    }
    print("%\n", xs);
}
