trace: int;
inner :: (selected: bool) #expand {
    if selected {
        defer trace = trace * 10 + 3;
        `return 42;
    }
}
outer :: (selected: bool) #expand {
    defer trace = trace * 10 + 2;
    inner(selected);
    trace = 99;
}
answer :: (selected: bool) -> int {
    defer trace = trace * 10 + 1;
    outer(selected);
    return 0;
}
main :: () -> int {
    result := answer(true);
    if trace != 321 return 1;
    return result;
}
