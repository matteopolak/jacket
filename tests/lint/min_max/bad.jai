#import "Basic";

main :: () {
    x := 50;
    a := min(0, max(100, x));
    b := max(100, min(x, 0));
    print("% %\n", a, b);
}
