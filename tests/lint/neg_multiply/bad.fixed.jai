#import "Basic";

main :: () {
    x := 5;
    y := 3;
    f := 1.5;

    a := -x;
    b := -y;
    c := -f;
    d := -(x + y);
    e := -(2 * x);
    g := -x + y;

    print("% % % % % %\n", a, b, c, d, e, g);
}
