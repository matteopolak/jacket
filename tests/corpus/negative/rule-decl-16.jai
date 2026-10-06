// A multi-value constant declaration cannot name more values than the expression has.
// rules: decl.16
two :: () -> int, int { return 1, 2; }
A, B, C :: two();
main :: () {
    x := A + B + C;
}
