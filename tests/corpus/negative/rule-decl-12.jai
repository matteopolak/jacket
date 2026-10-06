// A nested procedure cannot read a runtime local of its enclosing procedure: it is not a closure.
// rules: decl.12
main :: () {
    local := 5;
    inner :: () -> int { return local; }
    inner();
}
