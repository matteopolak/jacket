trace: int;
next :: (digit: int) -> int {
    trace = trace * 10 + digit;
    return digit;
}
finish :: () #expand {
    `return right = next(2), left = next(4) * 10;
}
pair :: () -> (left: int #must, right: int, extra: int = 7) {
    finish();
}
main :: () -> int {
    left, right, extra := pair();
    if trace != 24 return 1;
    return left + right + extra - 7;
}
