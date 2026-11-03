#import "Basic";

main :: () {
    scratch := NewArray(1 << 20, u8);
    defer free(scratch.data);
    scratch[0] = 1;

    print("%\n", scratch[0]);
}
