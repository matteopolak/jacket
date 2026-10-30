// A `$words: [] T` parameter given an array literal or a constant array binds a view over it,
// for runtime code and for `#run` alike.
#import "Basic";

total_length :: (words: [] string) -> int {
    total := 0;

    for words total += it.count;

    return total;
}

sum :: (values: [] int) -> int {
    total := 0;

    for values total += it;

    return total;
}

at_runtime :: ($values: [] int) -> int {
    return sum(values);
}

at_compile_time :: ($words: [] string) -> int {
    return #run total_length(words);
}

NAMES :: string.["if", "else", "while"];

main :: () {
    assert(at_runtime(int.[1, 2, 3]) == 6);

    LIMITS :: int.[4, 5];

    assert(at_runtime(LIMITS) == 9);
    assert(at_compile_time(string.["if", "else", "while"]) == 11);
    assert(at_compile_time(NAMES) == 11);

    print("ok\n");
}
