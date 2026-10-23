// A float literal written without a leading zero still parses after `cast(...)`.
#import "Basic";

main :: () {
    x := cast(float) .5 * 2;
    assert(x == 1);
    print("%\n", x);
}
