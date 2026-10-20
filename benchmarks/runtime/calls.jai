// Small procedure calls and recursion. Prints the results.
#import "Basic";

fib :: (n: int) -> int {
    if n < 2 return n;
    return fib(n - 1) + fib(n - 2);
}

ackermann :: (m: int, n: int) -> int {
    if m == 0 return n + 1;
    if n == 0 return ackermann(m - 1, 1);
    return ackermann(m - 1, ackermann(m, n - 1));
}

main :: () {
    print("% %\n", fib(34), ackermann(2, 3000));
}
