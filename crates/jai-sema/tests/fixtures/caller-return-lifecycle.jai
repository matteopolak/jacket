trace: int;
evaluations: int;
next :: () -> int {
    evaluations += 1;
    return 18 + evaluations * 2;
}
convert :: (first: int, second: int) #expand {
    result := first + second;
    defer { trace = trace * 10 + 3; result = 99; }
    `defer trace = trace * 10 + 2;
    {
        defer trace = trace * 10 + 4;
        `return result;
    }
}
answer :: () -> int #must {
    result := 100;
    defer trace = trace * 10 + 1;
    convert(next(), next());
}
main :: () -> int {
    result := answer();
    if evaluations != 2 return 1;
    if trace != 4321 return 2;
    return result;
}
