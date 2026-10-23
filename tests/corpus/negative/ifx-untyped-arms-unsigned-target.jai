#import "Basic";

main :: () {
    wide := true;
    base: u8 = ifx wide then 1 else 300;
    print("%\n", base);
}
