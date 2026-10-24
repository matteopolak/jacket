#import "Basic";

SCALE: float32 : 16777.217;

main :: () {
    pi: float32 = 31415.926;

    scaled := pi * 1234.5679;

    pi = -98765.43;

    big: float32 = 123456.79;

    print("% % % %\n", pi, scaled, big, SCALE);
}
