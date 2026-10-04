// A float literal is float32 unless it has more significant figures than float32 holds; fixed
// notation prints at most 18 significant digits.
#import "Basic";

PI_LONG :: 3.14159265358979;

main :: () {
    a := 12342345234.0;
    b := 2.25193;
    c := 16777216.0;
    #assert type_of(a) == float64;
    #assert type_of(b) == float32;
    #assert type_of(c) == float32;
    #assert type_of(PI_LONG) == float64;
    f: float32 = 2;
    g := f * 3.14159265358979;
    #assert type_of(g) == float32;
    h: float32 = PI_LONG;

    if tprint("%", a) != "12342345234" exit(1);
    if tprint("%", 5.97219e24) != "5972189887017193070000000" exit(2);
    big: float64 = 1e20;
    if tprint("%", big) != "100000000000000000000" exit(3);
    if tprint("%", 1e20) != "100000002004087734000" exit(5); // float32: 18 significant digits
    if tprint("%", h) != "3.141593" exit(4);
    print("ok\n");
}
