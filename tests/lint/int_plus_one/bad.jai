#import "Basic";

main :: () {
    x := 5;
    y := 3;
    n: u8 = 9;

    if x >= y + 1 print("a\n");
    if x - 1 >= y print("b\n");
    if y + 1 <= x print("c\n");
    if y <= x - 1 print("d\n");
    if n >= y + 1 print("e\n");
    if (x + y) - 1 >= n print("f\n");

    ok := (x >= (y + 1));

    print("%\n", ok);
}
