factorial :: (value: int) -> int {
    if value <= 1 return 1;
    return value * #this(value - 1);
}

main :: () -> int {
    return factorial(5) - 78;
}
