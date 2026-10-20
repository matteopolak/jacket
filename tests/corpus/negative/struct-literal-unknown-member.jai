// A named struct literal can only set members the struct has.
#import "Basic";
Color :: struct { r, g, b: float; }
main :: () {
    c := Color.{x = 1};
    print("%\n", c);
}
