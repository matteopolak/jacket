#import "Basic";

Pair :: struct {
    first: int;
    second: int;
}

main :: () {
    a := 1;
    b := 2;
    a = b;
    b = a;
    p: Pair;
    p.first = p.second;
    p.second = p.first;
    print("% % %\n", a, b, p);
}
