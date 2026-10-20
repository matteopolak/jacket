#import "Basic";

A :: struct {
    model: int;
}

B :: struct {
    model: int;
}

main :: () {
    a: A;
    b: B;
    using a;
    using b;
    print("%\n", model);
}
