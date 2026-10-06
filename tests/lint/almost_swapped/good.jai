#import "Basic";

main :: () {
    a := 1;
    b := 2;
    a, b = b, a;
    t := a;
    a = b;
    b = t;
    c := a;
    a = c;
    print("% % %\n", a, b, c);
}
