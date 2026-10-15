#import "Basic";

// `$a: [$N] float` bound to an array literal argument infers N from the literal.
sum_baked :: ($a: [$N] float) -> float, s64 {
    total := 0.0;
    for a total += it;
    return total, N;
}

main :: () {
    total, n := sum_baked(.[1.0, 2.0, 3.0, 4.0]);
    assert(n == 4, "N is %", n);
    assert(total == 10.0, "total is %", total);
}
