#import "Basic";

main :: () {
    items := int.[1, 2, 3, 4];
    total := 0;

    for items {
        if it == 2 continue;
        total += it;
    }

    for items {
        if it > 2 {
            total += it;
            continue;
        }

        total -= 1;
    }

    for items {
        for 0..it {
            if it_index > 1 continue;
            total += 1;
        }
    }

    i := 0;
    while: outer i < 10 {
        i += 1;

        for items {
            if it == 3 continue outer;
            total += it;
        }
    }

    print("%\n", total);
}
