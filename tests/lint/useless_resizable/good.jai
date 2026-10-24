#import "Basic";

take :: (xs: [..] int) -> int {
    return xs.count;
}

fill :: (xs: *[..] int) {
    array_add(xs, 1);
}

main :: (n: int) {
    // Added in a loop: the count is not known.
    looped: [..] int;

    for 0..n {
        array_add(*looped, it);
    }

    // Added under a condition.
    maybe: [..] int;

    if n > 3 {
        array_add(*maybe, 1);
    }

    array_add(*maybe, 2);

    // Handed to something that may need the resizable type.
    handed: [..] int;

    array_add(*handed, 1);

    count := take(handed);

    // Its address escapes.
    escaped: [..] int;

    array_add(*escaped, 1);
    fill(*escaped);

    // Never filled here.
    empty: [..] int;

    print("% % % % %\n", looped.count, maybe.count, count, escaped.count, empty.count);
}
