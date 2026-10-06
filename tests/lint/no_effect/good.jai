#import "Basic";

main :: () {
    x := 1;
    x = 5;
    x += 1;
    print("%\n", x);
    y := #run 1 + 2;
    print("%\n", y);
}
