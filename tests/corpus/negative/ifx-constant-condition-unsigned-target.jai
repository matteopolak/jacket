#import "Basic";

main :: () {
    c: u8 = ifx true then 1 else 256;
    print("%\n", c);
}
