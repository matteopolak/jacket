#import "Basic";

main :: () {
    wide := true;
    base: u64 = ifx wide then 1024 else 1000;
    print("%\n", base);
}
