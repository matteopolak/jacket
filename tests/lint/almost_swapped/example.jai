#import "Basic";

main :: () {
    a := 1;
    b := 2;

    a = b;
    b = a;

    print("% %\n", a, b);
}
