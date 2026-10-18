// A `[$N] T` parameter takes N from the element count of an untyped `.[...]` argument, or
// from the default when the argument is omitted.
#import "Basic";

count_of :: (tags: [$N] string) -> int {
    return N;
}

count_default :: (tags: [$N] string = .["a", "b"]) -> int {
    return N;
}

sum_of :: (values: [$N] int) -> int {
    total := 0;
    for values total += it;
    return total + N * 100;
}

main :: () {
    if count_of(.["m1", "m2"]) != 2 exit(1);
    if count_of(.["a", "b", "c"]) != 3 exit(2);
    if count_default(.["x", "y"]) != 2 exit(3);
    if count_default() != 2 exit(4);
    if count_default(.["x", "y"]) != 2 exit(5);
    if sum_of(.[1, 2, 3]) != 306 exit(6);
}
