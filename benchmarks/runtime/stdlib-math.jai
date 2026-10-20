// Math: square roots, trigonometry and rounding over a range of inputs.
#import "Basic";
#import "Math";

main :: () {
    sum: float64;

    for 0..10_000_000 - 1 {
        x := cast(float64) it * 0.0001;
        sum += sqrt(x) + sin(x) * cos(x) + abs(x - 500) + floor(x) + atan2(x, 3.0) + exp(-x) + log(x + 1);
    }

    print("%\n", cast(s64) (sum * 1000));
}
