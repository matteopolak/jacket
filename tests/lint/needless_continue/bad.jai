#import "Basic";

main :: () {
    items := int.[1, 2, 3, 4];
    total := 0;

    for items {
        total += it;
        continue;
    }

    for items {
        if it > 2 {
            total += it;
            continue;
        } else {
            total -= 1;
            continue;
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
            continue;
        }
    }

    print("%\n", total);
}
