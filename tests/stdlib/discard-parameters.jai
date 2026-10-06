// `#discard` parameters: the argument is typechecked but never evaluated, and the callee cannot name
// the parameter. Basic's assert uses this when ENABLE_ASSERT is off.
#import "Basic";

calls: int;
side :: () -> bool {
    calls += 1;
    return true;
}
side_int :: () -> int {
    calls += 1;
    return 7;
}

check :: (#discard cond: bool) {}
two :: (a: int, #discard b: int, c: int) -> int {
    return a + c;
}
labelled :: (#discard text := "none", #discard rest: ..Any) -> int {
    return 5;
}
pick :: (#discard x: $T, y: T) -> T {
    return y;
}

twice :: (#discard v: int) #expand {
    `calls += 100;
}

main :: () {
    check(side());
    if calls != 0 exit(1);
    if two(1, side_int(), 2) != 3 exit(2);
    if calls != 0 exit(3);
    if labelled(tprint("%", side_int()), side_int()) != 5 exit(4);
    if calls != 0 exit(5);
    if pick(side_int(), 9) != 9 exit(6);
    if calls != 0 exit(7);
    twice(side_int());
    if calls != 100 exit(8);
    if two(b = side_int(), a = 4, c = 5) != 9 exit(9);
    if calls != 100 exit(10);
    print("ok\n");
}
