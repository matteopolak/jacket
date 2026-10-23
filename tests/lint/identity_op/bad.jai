#import "Basic";

main :: () {
    x := 7;
    a := x + 0;
    b := 1 * x;
    c := x - 0;
    d := (x | 0) * 2;
    e := (x + x + 0) * 3;

    print("% % % % %\n", a, b, c, d, e);
}
