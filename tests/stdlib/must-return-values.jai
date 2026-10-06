// `#must` results can be used, assigned, returned or passed on; only discarding the call is an error
// (see tests/corpus/negative/must-*.jai). Calls in macro arguments may violate it until code is generated.
// rules: must.5 must.6 must.8
#import "Basic";

parse :: (x: int) -> int, bool #must {
    return x * 2, x > 0;
}
only :: (x: int) -> int #must {
    return x + 1;
}
macro_only :: (x: int) -> int #must #expand {
    return x + 2;
}
lenient :: (x: int) -> int {
    return x;
}
ignore :: (code: Code) #expand {}

main :: () {
    a, ok := parse(4);
    if a != 8 || !ok exit(1);
    b := only(1);
    if b != 2 exit(2);
    _ := only(5);
    _, _ := parse(3);
    if only(2) + macro_only(1) != 6 exit(3);
    lenient(4);
    ignore(only(1));
    c: int;
    c = macro_only(c);
    if c != 2 exit(4);
    print("ok\n");
}
