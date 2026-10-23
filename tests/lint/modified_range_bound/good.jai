#import "Basic";

first_zero :: (xs: [] int) -> int {
    n := xs.count;
    for 0..n - 1 {
        if xs[it] == 0 {
            n = it;
            break;
        }
    }

    return n;
}

count_down :: (start: int) -> int {
    n := start;
    total := 0;

    while n > 0 {
        total += n;
        n -= 1;
    }

    return total;
}

other_variable :: (xs: [] int) -> int {
    n := xs.count;
    kept := n;

    for 0..n - 1 {
        if xs[it] == 0 kept -= 1;
    }

    return kept;
}

main :: () {
    print("%\n", first_zero(.[1, 0, 2]));
    print("%\n", count_down(3));
    print("%\n", other_variable(.[1, 0, 2]));
}
