// Portable compiler control for the mixed native System/Process regressions.
// Those original programs still require real processor queries/child processes.
#import "Basic";

Mode :: enum {
    slow;
    fast;
}

speed :: ($mode: Mode = .fast) -> int {
    return ifx mode == .fast then 2 else 1;
}

factorial :: (value: int) -> int {
    if value <= 1 return 1;
    return value * factorial(value - 1);
}

COMPILE_PHASE :: #run -> bool {
    return #compile_time;
};

PRODUCT :: #run factorial(7);
#assert COMPILE_PHASE && PRODUCT == 5040;
#assert speed() == 2 && speed(.slow) == 1;

main :: () {
    assert(!#compile_time);
    assert(speed() == 2 && speed(.slow) == 1);
    assert(PRODUCT == factorial(7));
    print("ok\n");
}
