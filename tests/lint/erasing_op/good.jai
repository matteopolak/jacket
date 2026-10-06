#import "Basic";

STRIDE :: 0;

main :: () {
    x := 7;
    m: [16] int;
    row := 1;
    a := m[0 * 4 + row]; // lines up with the rows below
    b := x * STRIDE; // a named constant may change
    c := 0.0 * cast(float) x;
    print("% % % %\n", a, b, c, x * 2);
}
