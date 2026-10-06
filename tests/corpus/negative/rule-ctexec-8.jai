// rules: ctexec.8
// Unbounded recursion inside #run stops with a stack overflow error instead of hanging.
descend :: (n: int) -> int {
    return descend(n + 1) + 1;
}

DEPTH :: #run descend(0);

main :: () {
    x := DEPTH;
}
