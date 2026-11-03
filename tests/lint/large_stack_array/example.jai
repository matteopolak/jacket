#import "Basic";

main :: () {
    scratch: [1 << 20] u8;
    scratch[0] = 1;

    print("%\n", scratch[0]);
}
