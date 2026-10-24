#import "Basic";

SCALE: float32 : 16777.2171234;

main :: () {
    pi: float32 = 31415.92653;

    scaled := pi * 1234.5678901;

    pi = -98765.43210987;

    big: float32 = 123456.789012;

    print("% % % %\n", pi, scaled, big, SCALE);
}
