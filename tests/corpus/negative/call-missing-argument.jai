// A call must supply every argument that has no default.
#import "Basic";
f :: (a: int, b: int) -> int { return a + b; }
main :: () {
    print("%\n", f(1));
}
