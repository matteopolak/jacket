order: int;
register :: (digit: int) #expand {
    captured := digit;
    `defer order = order * 10 + captured;
}

work :: () -> int {
    register(2);
    register(1);
    return 12;
}

main :: () -> int { return work() + order + 18; }
