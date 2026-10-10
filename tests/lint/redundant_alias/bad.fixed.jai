#import "Basic";

square :: (n: int) -> int {
    return n * n;
}

one :: () -> int {
    return 7;
}

called :: () -> int {
    return one();
}

copies :: (n: int) {
    total := n * 3;
    total += 1;
    print("%\n", total);

    // A literal copied, then changed.
    count := 10;
    count += n;
    print("%\n", count);

    // Multi-line initializer.
    area := (n + 1) *
        (n + 2);
    print("%\n", area);
}

// An explicit type on the first is kept.
typed :: () {
    big: u8 = 5;
    print("%\n", big);
}

in_loop :: () {
    for 0..2 {
        shown := it * 2;
        print("%\n", shown);
    }
}

main :: () {
    copies(4);
    typed();
    in_loop();
    print("%\n", square(3) + called());
}
