grow :: (xs: *[..] int) {} @GrowsArray
consume :: (p: *int) -> int {
    return p.*;
}

alternatives :: (first: bool, second: bool) {
    xs: [..] int;
    p := *xs[0];

    if first grow(*xs);
    else if second grow(*xs);

    consume(p);
}

repeated :: (again: bool) {
    xs: [..] int;
    p := *xs[0];
    while again grow(*xs);
    consume(p);
}
