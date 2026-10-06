// Variables are declared left to right: a local cannot be used before its declaration.
// rules: decl.2
main :: () {
    y := x + 1;
    x := 2;
}
