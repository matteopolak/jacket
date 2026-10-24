#import "Basic";

Vec :: struct {
    x: float;
}

operator * :: (v: Vec, s: float) -> Vec {
    return .{ x = v.x * s * 2.0 };
}

main :: () {
    x := 5;
    n: u32 = 3;
    f := 1.5;
    v: Vec;

    a := -x;
    b := x * -2;
    c := x * 1;
    d := f * -1.5;
    e := n * 4;
    w := v * -1.0; // an overloaded operator decides what this means
    h := -1 + x;

    print("% % % % % % %\n", a, b, c, d, e, w, h);
}
