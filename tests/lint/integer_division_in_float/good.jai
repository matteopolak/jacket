#import "Basic";

main :: () {
    done := 3;
    total := 4;
    w := 640;
    ratio := cast(float) done / total;
    center := cast(float) (w / 2); // whole pixels on purpose
    pages := done / total;
    print("% % %\n", ratio, center, pages);
}
