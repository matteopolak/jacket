#import "Basic";

main :: () {
    p := cast(*u8) alloc(8);
    p[0] = 42;
    assert(p[0] == 42);
    free(p);
}
