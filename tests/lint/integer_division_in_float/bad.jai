#import "Basic";

main :: () {
    done := 3;
    total := 4;
    ratio := cast(float) (done / total);
    width := 10.0;
    half := 1 / 2 * width;

    sum := (done + total) / total * 0.5;

    print("% % %\n", ratio, half, sum);
}
