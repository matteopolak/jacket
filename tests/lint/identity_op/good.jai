#import "Basic";

OFFSET :: 0;

main :: () {
    x := 7;
    data: [8] int;
    a := data[x - 7 + 0]; // lines up with neighbouring indices
    b := x + OFFSET; // a named constant may change
    c := 1.0 * cast(float) x;
    d := ((x >> 0) & 0xF) | ((x >> 4) & 0xF); // lines up with the other shifts
    print("% % % %\n", a, b, c, d);
}
