#import "Basic";

average :: (values: [] float64) -> float64 {
    total: float64;
    for values total += it;
    return total / cast(float64) values.count;
}

digits :: (n: s64) -> s64 {
    count: s64;

    while n > 0 {
        n = n / 10;
        count += 1;
    }

    return count;
}

main :: () {
    values := float64.[1, 2, 3];
    print("% %\n", average(values), digits(12345));
}
