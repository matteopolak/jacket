#import "Basic";

main :: () {
    x := 5;
    y := 3;
    f := 1.5;
    g := 2.5;

    if x > y print("a\n");
    if x >= y + 2 print("b\n"); // not one
    if x - 1 > y print("c\n"); // already strict
    if x + 1 >= y print("d\n"); // `+ 1` on the larger side keeps the equality
    if x - 1 <= y print("e\n");
    if f >= g + 1 print("f\n"); // floats are not stepped by one
    if x >= y - 1 print("g\n"); // not a strict comparison
    if x + y >= y + x print("h\n");
}
