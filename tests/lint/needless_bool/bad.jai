#import "Basic";

positive :: (n: int) -> bool {
    if n > 0 return true;
    else return false;
}

small :: (n: int) -> bool {
    if n < 10 {
        return false;
    }

    return true;
}

differs :: (a: int, b: int) -> bool {
    if a == b return false;
    return true;
}

// `V` only defines `operator ==`, so `!=` would not compile.
V :: struct {
    x: int;
}

operator == :: (a: V, b: V) -> bool {
    return a.x == b.x;
}

distinct_values :: (a: V, b: V) -> bool {
    if a == b return false;
    return true;
}

main :: () {
    n := 5;
    even := ifx n % 2 == 0 then true else false;
    odd := ifx even then false else true;
    flag: bool;

    if n > 3 flag = true;
    else flag = false;

    print("% % % % %\n", positive(n), small(n), even, odd, flag);
}
