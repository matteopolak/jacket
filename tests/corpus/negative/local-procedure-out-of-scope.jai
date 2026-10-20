// A local procedure is invisible outside its enclosing procedure.
#import "Basic";
analyze :: () {
    cube :: (n: int) -> int { return n * n * n; }
    print("%\n", cube(4));
}
main :: () {
    analyze();
    print("%\n", cube(4));
}
