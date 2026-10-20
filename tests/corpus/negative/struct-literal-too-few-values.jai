#import "Basic";

Color :: struct {
    r, g, b: float;
}

main :: () {
    c := Color.{0.2, 0.5};
    print("%\n", c.r);
}
