#import "Basic";

positive :: (n: int) -> bool {
    return n > 0;
}

small :: (n: int) -> bool {
    return n >= 10;
}

main :: () {
    n := 5;
    even := n % 2 == 0;
    odd := !even;
    flag: bool;
    flag = n > 3;
    print("% % % % %\n", positive(n), small(n), even, odd, flag);
}
