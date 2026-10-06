#import "Basic";

main :: () {
    done := 3;
    total := 4;
    ratio := cast(float) (done / total);
    width := 10.0;
    half := 1 / 2 * width;
    print("% %\n", ratio, half);
}
