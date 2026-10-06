#import "Basic";

main :: () {
    big := 70000;
    small: u16 = xx big;
    ratio := 2.75;
    whole: int = xx ratio;
    precise: float64 = 1.0 / 3.0;
    rough: float32 = xx precise;
    print("% % %\n", small, whole, rough);
}
