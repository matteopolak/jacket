calls: int;
total: int;

tick :: () -> int { calls += 1; return 7; }
ignore :: (#discard value: int) #expand {
    body :: #code { total = 42; };
    #insert body;
}

main :: () -> int {
    ignore(tick());
    ignore(#run tick());
    return total + calls * 100;
}
