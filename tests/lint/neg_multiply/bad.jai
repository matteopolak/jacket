#import "Basic";

main :: () {
    x := 5;
    y := 3;
    f := 1.5;

    a := x * -1;
    b := -1 * y;
    c := f * -1.0;
    d := (x + y) * -1;
    e := 2 * x * -1;
    g := x * -1 + y;

    print("% % % % % %\n", a, b, c, d, e, g);
}
