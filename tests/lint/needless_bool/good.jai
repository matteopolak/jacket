#import "Basic";

positive :: (n: int) -> bool {
    if n > 0 return true;
    print("not positive\n");
    return false;
}

// The last check of a series of guards stays a guard.
checked :: (n: int) -> bool {
    if n < 0 return false;
    half := n / 2;
    if half * 2 != n return false;
    return true;
}

main :: () {
    n := 5;
    yes := ifx n > 0 then 1 else 0;
    a: bool;
    b: bool;
    if n > 3 a = true;
    else b = false;
    if n > 2 {
        // keep this note
        return;
    }
    print("% % % % %\n", positive(n), checked(n), yes, a, b);
}
