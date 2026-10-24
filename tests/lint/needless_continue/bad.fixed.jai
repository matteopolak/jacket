#import "Basic";

main :: () {
    items := int.[1, 2, 3, 4];
    total := 0;

    for items {
        total += it;
    }

    for items {
        if it > 2 {
            total += it;
        } else {
            total -= 1;
        }
    }

    i := 0;
    while i < 10 {
        i += 1;
        if i % 2 == 0 continue;
    }

    for items {
        if it == 3 {
            total *= 2;
        } else if it == 4 {
            total += 3;
        }
    }

    print("%\n", total);
}
