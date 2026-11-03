#import "Basic";

main :: () {
    big := 70000;
    small: u16 = cast,trunc(u16) big;
    print("%\n", small);
}
