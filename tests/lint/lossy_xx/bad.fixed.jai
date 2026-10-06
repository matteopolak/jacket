#import "Basic";

main :: () {
    big := 70000;
    small: u16 = cast(u16) big;
    ratio := 2.75;
    whole: int = cast(s64) ratio;
    precise: float64 = 1.0 / 3.0;
    rough: float32 = cast(float32) precise;
    print("% % %\n", small, whole, rough);
}
