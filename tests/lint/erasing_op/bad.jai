#import "Basic";

main :: () {
    x := 7;
    flags: u32 = 3;
    a := x * 0;
    b := flags & 0;
    c := x % 1;
    print("% % %\n", a, b, c);
}
