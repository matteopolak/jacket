// Interpreter: recursive calls, fib(32), long enough that start-up is noise.
#import "Basic";

fib :: (n: int) -> int {
    if n < 2 return n;
    return fib(n - 1) + fib(n - 2);
}

main :: () {
    print("%\n", fib(32));
}
