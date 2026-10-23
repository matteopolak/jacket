#import "Basic";

positive :: (n: int) -> bool {
    return n > 0;
}

small :: (n: int) -> bool {
    return n >= 10;
}

differs :: (a: int, b: int) -> bool {
    return a != b;
}

// `V` only defines `operator ==`, so `!=` would not compile.
V :: struct {
    x: int;
}

operator == :: (a: V, b: V) -> bool {
    return a.x == b.x;
}

distinct_values :: (a: V, b: V) -> bool {
    return !(a == b);
}

main :: () {
    n := 5;
    even := n % 2 == 0;
    odd := !even;
    flag: bool;

    flag = n > 3;

    print("% % % % %\n", positive(n), small(n), even, odd, flag);
}
