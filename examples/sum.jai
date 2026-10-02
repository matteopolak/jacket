sum_to :: (n: int) -> int {
    total := 0;
    while n > 0 {
        total += n;
        n -= 1;
    }
    return total;
}

main :: () -> int {
    return sum_to(9); // Exit status: 45
}
