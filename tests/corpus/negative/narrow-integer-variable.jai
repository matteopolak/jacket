// A variable of a wider integer type does not convert to `u8` without a cast.
#import "Basic";
main :: () {
    c := 10;
    d: u8 = c;
    print("%", d);
}
