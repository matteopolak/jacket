// A `float32` operand meeting a float literal that needs `float64` widens the operation to `float64`,
// for variables and for constants alike.
// rules: num.22
#import "Basic";
#import "Math";

CUTOFF: float : 70.0 / 44100.0;
K: float : cast(float) (CUTOFF / (CUTOFF + 0.1591549431));

main :: () {
    x: float = 1;
    y := x + 0.1591549431;

    assert(type_of(y) == float64);
    s := 0.1591549431 + x;

    assert(type_of(s) == float64);
    v := max(x, 10 ^ -40);

    assert(v == 1);
    assert(K > 0.00987 && K < 0.00988);
    print("ok\n");
}
