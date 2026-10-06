// rules: lambda.7
// A lambda does not capture the locals of the procedure it is written in.
apply :: (f: (int) -> int) -> int { return f(1); }
main :: () {
    n := 5;
    r := apply(x => x + n);
}
