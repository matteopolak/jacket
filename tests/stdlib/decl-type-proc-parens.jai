// A parenthesized list as a declaration type is a procedure type: `f: (*T);`.
#import "Basic";

Collector :: struct ($T: Type) {
    destroy: (*T);
}

total := 0;
add :: (x: *int) { total += x.*; }

main :: () {
    c: Collector(int);
    c.destroy = add;
    n := 5;
    c.destroy(*n);
    g: (int) -> int = (x: int) -> int { return x * 2; };
    if total != 5 || g(4) != 8 exit(1);
    print("ok\n");
}
