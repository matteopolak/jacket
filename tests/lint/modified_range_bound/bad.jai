#import "Basic";

trim :: (xs: [] int) -> int {
    n := xs.count;
    for 0..n - 1 {
        if xs[it] == 0 n -= 1;
    }

    return n;
}

grow :: (limit: int) -> int {
    end := limit;
    total := 0;

    for start: 1..end {
        total += start;
        end = total;
    }

    return total;
}

main :: () {
    print("%\n", trim(.[1, 0, 2]));
    print("%\n", grow(3));
}
