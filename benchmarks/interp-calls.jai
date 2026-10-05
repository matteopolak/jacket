// Interpreter: recursive calls and integer arithmetic.
#import "Basic";

fib :: (n: int) -> int {
    if n < 2 return n;
    return fib(n - 1) + fib(n - 2);
}

main :: () {
    print("%\n", fib(27));
}
