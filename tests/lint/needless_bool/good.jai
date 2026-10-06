#import "Basic";

positive :: (n: int) -> bool {
    if n > 0 return true;
    print("not positive\n");
    return false;
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
    print("% % % %\n", positive(n), yes, a, b);
}
