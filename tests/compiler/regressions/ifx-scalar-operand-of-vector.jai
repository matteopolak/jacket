// An `ifx` of untyped number constants that is an operand of a vector operator is a number, not
// the vector on the other side.
// rules: flow.32
#import "Basic";
#import "Math";

main :: () {
    base := Vector4.{ 0.5, 0.5, 0.5, 1 };
    lit := 1;

    scaled := base * (ifx lit != 0 then 0.9 else 1.0);

    assert(scaled.x > 0.44 && scaled.x < 0.46);

    picked := (ifx lit == 1 then base else base * 2) * (ifx lit != 0 then 0.9 else 1.0);

    assert(picked.w > 0.89 && picked.w < 0.91);

    print("ok\n");
}
