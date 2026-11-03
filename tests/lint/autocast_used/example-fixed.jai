#import "Basic";

main :: () {
    big: s64 = 200;
    small := cast(u8) big;

    print("%\n", small);
}
