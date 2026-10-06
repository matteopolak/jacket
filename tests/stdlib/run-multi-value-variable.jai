// rules: decl.17
// `x, y, z := #run f();` keeps every value of f, as the constant form does.
#import "Basic";

triple :: () -> int, int, int {
    return 1, 2, 3;
}

main :: () {
    x, y, z := #run triple();
    assert(x == 1 && y == 2 && z == 3, "#run keeps every value in a variable declaration");
    u, v := #run triple();
    assert(u == 1 && v == 2, "#run with fewer names takes the leading values");
    print("ok\n");
}
