leaf :: (n: int) -> int {
    return n + 1;
}

middle :: (n: int) -> int {
    doubled := leaf(n) * 2;
    return doubled + leaf(doubled);
}

main :: () {
    middle(3);
}
