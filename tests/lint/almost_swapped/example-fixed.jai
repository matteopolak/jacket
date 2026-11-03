#import "Basic";

main :: () {
    a := 1;
    b := 2;

    tmp := a;

    a = b;
    b = tmp;

    print("% %\n", a, b);
}
